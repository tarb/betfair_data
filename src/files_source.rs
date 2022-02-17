use bzip2_rs::decoder::ParallelDecoderReader;
use crossbeam_channel::{bounded, Receiver};
use flate2::bufread::GzDecoder;
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};
use pyo3::PyIterProtocol;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::thread;
// use std::io::{BufReader};

use crate::deser::DeserializerWithData;
use crate::{IOErr, MarketSource, SourceConfig, SourceItem};

#[pyclass]
pub struct Files {
    sources: FilesSource,
    config: SourceConfig,
}

impl MarketSource for Files {
    fn config(&self) -> SourceConfig {
        self.config
    }
    fn get(&mut self) -> Option<Result<SourceItem, IOErr>> {
        self.sources.chan.recv().ok()
    }
}

#[pymethods]
impl Files {
    #[new]
    #[args(cumulative_runner_tv = "true", stable_runner_index = "true")]
    fn __new__(
        paths: &PySequence,
        cumulative_runner_tv: bool,
        stable_runner_index: bool,
    ) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
            stable_runner_index,
        };

        let sources = (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .filter_map(|s| s.ok())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        let fs = FilesSource::new(sources)?;

        Ok(Self {
            config,
            sources: fs,
        })
    }
}

#[pyproto]
impl<'p> PyIterProtocol for Files {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        MarketSource::next(slf)
    }
}

struct FilesSource {
    chan: Receiver<Result<SourceItem, IOErr>>,
}

impl FilesSource {
    fn new(paths: Vec<PathBuf>) -> Result<Self, Error> {
        let (data_send, data_recv) = bounded(30);

        thread::spawn(move || -> Result<(), Error> {
            let _ = paths
                .into_par_iter()
                .map(|path| {
                    let buf: Result<Vec<u8>, Error> = try {
                        let mut file = File::open(&path)?;
                        let file_size = file.metadata()?.len();
                        // let mut bf_file = BufReader::new(file);

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

            Ok(())
        });

        Ok(Self { chan: data_recv })
    }
}
