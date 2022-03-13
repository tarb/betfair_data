use bzip2_rs::decoder::ParallelDecoderReader;
use crossbeam_channel::{bounded, Receiver};
use flate2::bufread::GzDecoder;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};
use std::fs::File;
use std::io::Read;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use crate::bflw::file_iter::BflwAdapter;
use crate::deser::DeserializerWithData;
use crate::errors::IOErr;
use crate::immutable::file_iter::ImmutAdapter;
use crate::market_source::{MarketSource, SourceConfig, SourceItem};
use crate::mutable::file_iter::MutAdapter;

#[pyclass]
pub struct Files {
    source: Option<FilesSource>,
}

#[pymethods]
impl Files {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    fn __new__(paths: &PySequence, cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
        };

        let sources = (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .filter_map(|s| s.ok())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        let fs = FilesSource::new(sources, config)?;

        Ok(Self { source: Some(fs) })
    }

    #[pyo3(name = "iter")]
    #[args(mutable = "false")]
    fn iter_adapter(&mut self, py: Python, mutable: bool) -> PyResult<PyObject> {
        let source = self.source.take();

        match source {
            Some(s) if mutable => Ok(MutAdapter::new(Box::new(s)).into_py(py)),
            Some(s) => Ok(ImmutAdapter::new(Box::new(s)).into_py(py)),

            None => Err(PyRuntimeError::new_err("empty source")),
        }
    }

    #[pyo3(name = "bflw")]
    fn bflw_adapter(&mut self) -> PyResult<BflwAdapter> {
        let source = self.source.take();

        match source {
            Some(s) => Ok(BflwAdapter::new(Box::new(s))),
            None => Err(PyRuntimeError::new_err("empty source")),
        }
    }
}

struct FilesSource {
    chan: Receiver<Result<SourceItem, IOErr>>,
    config: SourceConfig,
}

impl MarketSource for FilesSource {
    fn config(&self) -> SourceConfig {
        self.config
    }
}

impl Iterator for FilesSource {
    type Item = Result<SourceItem, IOErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chan.recv().ok()
    }
}

impl FilesSource {
    fn new(paths: Vec<PathBuf>, config: SourceConfig) -> Result<Self, Error> {
        const BUFFER_SIZE: usize = 50;
        let (data_send, data_recv) = bounded(BUFFER_SIZE);

        rayon::spawn(move || {
            let _ = paths
                .into_iter()
                .map(|path| {
                    let buf: Result<Vec<u8>, Error> = try {
                        let mut file = File::open(&path)?;
                        let file_size = file.metadata()?.len();

                        let mut buf: Vec<u8> = Vec::with_capacity(file_size as usize);
                        file.read_to_end(&mut buf)?;

                        buf
                    };

                    match buf {
                        Err(err) => Err(IOErr {
                            file: Some(path),
                            err,
                        }),

                        Ok(buf) => {
                            let r = match path.extension().and_then(|s| s.to_str()) {
                                Some("gz") => {
                                    let mut dec = GzDecoder::new(&buf[..]);
                                    let mut out_buf = Vec::with_capacity(buf.len());
                                    dec.read_to_end(&mut out_buf).map(|_| out_buf)
                                }
                                Some("bz2") => {
                                    let mut dec = ParallelDecoderReader::new(
                                        &buf[..],
                                        bzip2_rs::RayonThreadPool,
                                        1024 * 1024,
                                    );
                                    let mut out_buf = Vec::with_capacity(buf.len());
                                    dec.read_to_end(&mut out_buf).map(|_| out_buf)
                                }
                                Some("json") => Ok(buf),
                                Some(ext) => Err(Error::new(
                                    ErrorKind::Unsupported,
                                    format!("unsupported extension {}", ext),
                                )),
                                None => Err(Error::new(
                                    ErrorKind::Unsupported,
                                    "missing file extension",
                                )),
                            }
                            .and_then(DeserializerWithData::build);

                            match r {
                                Ok(deser) => Ok(SourceItem::new(path, deser)),
                                Err(err) => Err(IOErr {
                                    file: Some(path),
                                    err,
                                }),
                            }
                        }
                    }
                })
                .try_for_each(|r| data_send.send(r));
        });

        Ok(Self {
            chan: data_recv,
            config,
        })
    }
}
