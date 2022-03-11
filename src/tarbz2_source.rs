use bzip2_rs::decoder::ParallelDecoderReader;
use crossbeam_channel::{bounded, Receiver};
use ouroboros::self_referencing;
use pyo3::exceptions;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::PathBuf;
use tar::Archive;
use tar::Entries;

use crate::bflw::adapter::BflwAdapter;
use crate::deser::DeserializerWithData;
use crate::errors::IOErr;
use crate::immutable::adapter::ImmutAdapter;
use crate::market_source::{MarketSource, SourceConfig, SourceItem};
use crate::mutable::adapter::MutAdapter;

#[pyclass]
pub struct TarBz2 {
    source: Option<TarBzSource>,
}

#[pymethods]
impl TarBz2 {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    fn __new__(paths: &PySequence, cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
        };

        let source = (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .filter_map(|s| s.ok())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        let t = TarBzSource::new(source, config).map_err(|op: std::io::Error| {
            PyErr::new::<exceptions::PyRuntimeError, _>(op.to_string())
        })?;

        Ok(Self { source: Some(t) })
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

struct TarBzSource {
    chan: Receiver<Result<SourceItem, IOErr>>,
    config: SourceConfig,
}

impl MarketSource for TarBzSource {
    fn config(&self) -> SourceConfig {
        self.config
    }
}

impl Iterator for TarBzSource {
    type Item = Result<SourceItem, IOErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chan.recv().ok()
    }
}

impl TarBzSource {
    fn new(paths: Vec<PathBuf>, config: SourceConfig) -> Result<Self, Error> {
        let (data_send, data_recv) = bounded(paths.len() * 10);

        rayon::spawn(move || {
            let _ = paths
                .into_iter()
                .map(|path| (File::open(&path), path))
                .filter_map(|(file, path)| file.ok().map(|file| (file, path)))
                .flat_map(|(file, path)| TarEntriesIter::build(path, file))
                .map(|x| {
                    x.and_then(|(name, buf)| {
                        let mut out_buf = Vec::with_capacity(buf.len());

                        let r = ParallelDecoderReader::new(
                            &buf[..],
                            bzip2_rs::RayonThreadPool,
                            1024 * 1024,
                        )
                        .read_to_end(&mut out_buf)
                        .and_then(|_| DeserializerWithData::build(out_buf));

                        match r {
                            Ok(deser) => Ok(SourceItem::new(name, deser)),
                            Err(err) => Err(IOErr {
                                file: Some(name),
                                err,
                            }),
                        }
                    })
                })
                .try_for_each(|r: Result<SourceItem, IOErr>| data_send.send(r));
        });

        Ok(Self {
            chan: data_recv,
            config,
        })
    }
}

#[self_referencing]
struct TarEntriesIter {
    path: PathBuf,
    archive: Archive<File>,

    #[borrows(mut archive)]
    #[not_covariant]
    entries: Entries<'this, File>,
}

impl TarEntriesIter {
    fn build(path: PathBuf, file: File) -> Self {
        TarEntriesIterBuilder {
            path,
            archive: Archive::new(file),
            entries_builder: |archive| archive.entries().unwrap(),
        }
        .build()
    }
}

impl Iterator for TarEntriesIter {
    type Item = Result<(PathBuf, Vec<u8>), IOErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_mut(|slf| {
            loop {
                match slf.entries.next() {
                    Some(Ok(mut entry)) if entry.size() > 0 => {
                        let mut buf = Vec::with_capacity(entry.size() as usize);

                        let name = match entry.path().map(|path| slf.path.join(path)) {
                            Ok(name) => name,
                            Err(err) => break Some(Err(IOErr { file: None, err })),
                        };

                        match entry.read_to_end(&mut buf) {
                            Ok(_) => break Some(Ok((name, buf))),
                            Err(err) => {
                                break Some(Err(IOErr {
                                    file: Some(name),
                                    err,
                                }))
                            }
                        }
                    }
                    Some(Err(err)) => break Some(Err(IOErr { file: None, err })),
                    None => break None,
                    _ => {} //repeat
                }
            }
        })
    }
}
