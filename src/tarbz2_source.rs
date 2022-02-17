use crossbeam_channel::{bounded, Receiver};
use ouroboros::self_referencing;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};
use pyo3::PyIterProtocol;
use rayon::prelude::*;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use tar::Archive;
use tar::Entries;
// use bzip2_rs::decoder::DecoderReader;
use bzip2_rs::decoder::ParallelDecoderReader;

use crate::deser::DeserializerWithData;
use crate::MarketSource;
use crate::SourceConfig;
use crate::{IOErr, SourceItem};

#[pyclass]
pub struct TarBz2 {
    sources: TarBzSource,
    config: SourceConfig,
}

impl MarketSource for TarBz2 {
    fn config(&self) -> SourceConfig {
        self.config
    }
    fn get(&mut self) -> Option<Result<SourceItem, IOErr>> {
        self.sources.chan.recv().ok()
    }
}

#[pymethods]
impl TarBz2 {
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

        let t = TarBzSource::new(sources).map_err(|op: std::io::Error| {
            PyErr::new::<exceptions::PyRuntimeError, _>(op.to_string())
        })?;

        Ok(Self { config, sources: t })
    }
}

#[pyproto]
impl<'p> PyIterProtocol for TarBz2 {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        MarketSource::next(slf)
    }
}

struct TarBzSource {
    chan: Receiver<Result<SourceItem, IOErr>>,
}

impl TarBzSource {
    fn new(paths: Vec<PathBuf>) -> Result<Self, Error> {
        let (data_send, data_recv) = bounded(paths.len() * 10);

        thread::spawn(move || -> Result<(), Error> {
            let _ = paths
                .into_par_iter()
                .map(|path| (File::open(&path), path))
                .filter_map(|(file, path)| file.ok().map(|file| (file, path)))
                .flat_map_iter(|(file, path)| TarEntriesIter::build(path, file))
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

            Ok(())
        });

        Ok(Self { chan: data_recv })
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
