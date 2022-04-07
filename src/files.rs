use bzip2_rs::decoder::ParallelDecoderReader;
use crossbeam_channel::{bounded, Receiver};
use flate2::bufread::GzDecoder;
use ouroboros::self_referencing;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};
use std::fs::File;
use std::io::Read;
use std::io::{Error, ErrorKind};
use std::iter;
use std::path::PathBuf;
use tar::Archive as TarArchive;
use tar::Entries;
use zip::ZipArchive;

use crate::config::Config;
use crate::deser::DeserializerWithData;
use crate::errors::IOErr;
use crate::immutable::file::File as ImmutFile;
use crate::market_source::{Adapter, SourceItem};
use crate::mutable::file::File as MutFile;

const NUM_BUFFERED: usize = 50;

enum FileType {
    Mutable(Adapter<Config, MutFile>),
    Immutable(Adapter<Config, ImmutFile>),
}

#[pyclass]
pub struct Files {
    adapter: FileType,
}

#[pymethods]
impl Files {
    #[new]
    #[args(cumulative_runner_tv = "true", mutable = "false")]
    fn __new__(paths: &PySequence, cumulative_runner_tv: bool, mutable: bool) -> PyResult<Self> {
        let config = Config {
            cumulative_runner_tv,
        };

        let paths = FilesSource::get_paths(paths);
        let fs = FilesSource::new(paths).map_err(|op: std::io::Error| {
            PyErr::new::<exceptions::PyRuntimeError, _>(op.to_string())
        })?;

        let adapter = match mutable {
            true => FileType::Mutable(Adapter::new(fs, config)),
            false => FileType::Immutable(Adapter::new(fs, config)),
        };

        Ok(Self { adapter })
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PyObject> {
        match &mut self.adapter {
            FileType::Mutable(a) => a.next().map(|f| f.into_py(py)),
            FileType::Immutable(a) => a.next().map(|f| f.into_py(py)),
        }
    }
}

pub struct FilesSource {
    chan: Receiver<Result<SourceItem, IOErr>>,
}

impl Iterator for FilesSource {
    type Item = Result<SourceItem, IOErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chan.recv().ok()
    }
}

impl FilesSource {
    pub fn new(paths: Vec<PathBuf>) -> Result<Self, Error> {
        let (data_send, data_recv) = bounded(NUM_BUFFERED);

        rayon::spawn(move || {
            let _ = paths
                .into_iter()
                .map(|path| (File::open(&path), path))
                .map(|(file, path)| match file {
                    Ok(f) => handle_file(path, f),
                    Err(err) => Err(IOErr {
                        file: Some(path),
                        err,
                    }),
                })
                .flat_map(|r| match r {
                    Ok(iter) => iter,
                    Err(err) => Box::new(iter::once(Err(err))),
                })
                .map(|x| x.and_then(|(name, buf)| handle_buffer(name, buf)))
                .try_for_each(|r: Result<SourceItem, IOErr>| data_send.send(r));
        });

        Ok(Self { chan: data_recv })
    }

    pub fn get_paths(paths: &PySequence) -> Vec<PathBuf> {
        (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .filter_map(|s| s.ok())
            .map(PathBuf::from)
            .collect::<Vec<_>>()
    }
}

type BoxedArchiveIter = Box<dyn Iterator<Item = Result<(PathBuf, Vec<u8>), IOErr>>>;

fn handle_file(path: PathBuf, mut file: File) -> Result<BoxedArchiveIter, IOErr> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("tar") => Ok(Box::new(TarEntriesIter::build(path, file))),
        Some("zip") => Ok(Box::new(ZipEntriesIter::try_build(path, file)?)),
        Some("json") | Some("gz") | Some("bz2") | None => {
            let buf = try {
                let file_size = file.metadata()?.len();
                let mut buf: Vec<u8> = Vec::with_capacity(file_size as usize);
                file.read_to_end(&mut buf)?;

                buf
            };

            match buf {
                Ok(buf) => Ok(Box::new(iter::once(Ok((path, buf))))),
                Err(err) => Err(IOErr {
                    file: Some(path),
                    err,
                }),
            }
        }
        Some(_) => Err(IOErr {
            err: Error::new(ErrorKind::Unsupported, "unsupported file type"),
            file: Some(path),
        }),
    }
}

fn handle_buffer(path: PathBuf, buf: Vec<u8>) -> Result<SourceItem, IOErr> {
    let r = match path.extension().and_then(|s| s.to_str()) {
        Some("gz") => {
            let mut dec = GzDecoder::new(&buf[..]);
            let mut out_buf = Vec::with_capacity(buf.len());
            dec.read_to_end(&mut out_buf).map(|_| out_buf)
        }
        Some("bz2") => {
            let mut dec =
                ParallelDecoderReader::new(&buf[..], bzip2_rs::RayonThreadPool, 1024 * 1024);
            let mut out_buf = Vec::with_capacity(buf.len());
            dec.read_to_end(&mut out_buf).map(|_| out_buf)
        }
        Some("json") | None => Ok(buf),
        Some(ext) => Err(Error::new(
            ErrorKind::Unsupported,
            format!("unsupported extension {}", ext),
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

#[self_referencing]
struct TarEntriesIter {
    path: PathBuf,
    archive: TarArchive<File>,

    #[borrows(mut archive)]
    #[not_covariant]
    entries: Entries<'this, File>,
}

impl TarEntriesIter {
    fn build(path: PathBuf, file: File) -> Self {
        TarEntriesIterBuilder {
            path,
            archive: TarArchive::new(file),
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

struct ZipEntriesIter {
    path: PathBuf,
    archive: ZipArchive<File>,
    len: usize,
    pos: usize,
}

impl ZipEntriesIter {
    fn try_build(path: PathBuf, file: File) -> Result<Self, IOErr> {
        match ZipArchive::new(file) {
            Ok(archive) => Ok(ZipEntriesIter {
                path,
                len: archive.len(),
                archive,
                pos: 0,
            }),
            Err(err) => Err(IOErr {
                file: Some(path),
                err: err.into(),
            }),
        }
    }
}

impl Iterator for ZipEntriesIter {
    type Item = Result<(PathBuf, Vec<u8>), IOErr>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.len {
                break None;
            }

            match self.archive.by_index(self.pos) {
                Ok(mut zfile) => {
                    self.pos += 1;

                    if zfile.is_dir() {
                        continue;
                    }

                    let name = self.path.join(zfile.mangled_name());

                    let mut buffer = Vec::with_capacity(zfile.size() as usize);
                    match zfile.read_to_end(&mut buffer) {
                        Ok(_s) => break Some(Ok((name, buffer))),
                        Err(err) => {
                            break Some(Err(IOErr {
                                file: Some(name),
                                err,
                            }))
                        }
                    }
                }
                Err(err) => {
                    self.pos += 1;

                    break Some(Err(IOErr {
                        file: None,
                        err: err.into(),
                    }));
                }
            }
        }
    }
}
