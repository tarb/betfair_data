use log::warn;
use pyo3::types::{PySequence, PyString};
use pyo3::{exceptions, prelude::*};
use serde::de::DeserializeSeed;
use std::path::PathBuf;

use super::config::{Config, ConfigBuilder};
use super::market_book::{MarketBook, MarketBooksDeser};
use crate::deser::DeserializerWithData;
use crate::files::FilesSource;
use crate::immutable::container::SyncObj;
use crate::market_source::{Adapter, SourceItem};

#[pyclass(name = "Files")]
pub struct BflwFiles {
    adapter: Adapter<ConfigBuilder, BflwFile>,
}

#[pymethods]
impl BflwFiles {
    #[new]
    #[args(cumulative_runner_tv = "true", streaming_unique_id = "None")]
    fn __new__(
        paths: &PySequence,
        cumulative_runner_tv: bool,
        streaming_unique_id: Option<u32>,
    ) -> PyResult<Self> {
        let config = ConfigBuilder {
            cumulative_runner_tv,
            streaming_unique_id,
        };

        let source = (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .filter_map(|s| s.ok())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        let fs = FilesSource::new(source).map_err(|op: std::io::Error| {
            PyErr::new::<exceptions::PyRuntimeError, _>(op.to_string())
        })?;

        let adapter = Adapter::new(fs, config);

        Ok(Self { adapter })
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PyObject> {
        self.adapter.next().map(|f| f.into_py(py))
    }
}

#[pyclass(name = "File")]
pub struct BflwFile {
    #[pyo3(get)]
    file_name: SyncObj<PathBuf>,
    config: Config,
    deser: Option<DeserializerWithData>,
    books: Vec<Py<MarketBook>>,
}

impl BflwFile {
    fn drive_deserialize(
        deser: &mut DeserializerWithData,
        books: &[Py<MarketBook>],
        config: Config,
        py: Python,
    ) -> Result<Vec<Py<MarketBook>>, serde_json::Error> {
        deser.with_dependent_mut(|_, deser| {
            MarketBooksDeser {
                markets: books,
                py,
                config,
            }
            .deserialize(&mut deser.0)
        })
    }
}

impl From<(SourceItem, Config)> for BflwFile {
    fn from(s: (SourceItem, Config)) -> Self {
        let (item, config) = s;

        Self {
            file_name: SyncObj::new(item.file),
            deser: Some(item.deser),
            books: Vec::new(),
            config,
        }
    }
}

#[pymethods]
impl BflwFile {
    #[new]
    #[args(cumulative_runner_tv = "true", streaming_unique_id = "None")]
    fn __new__(
        file: PathBuf,
        bytes: &[u8],
        cumulative_runner_tv: bool,
        streaming_unique_id: Option<u32>,
    ) -> PyResult<Self> {
        let config = Config {
            cumulative_runner_tv,
            streaming_unique_id,
        };

        let deser = DeserializerWithData::build(bytes.to_owned())
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        Ok(Self {
            file_name: SyncObj::new(file),
            deser: Some(deser),
            books: Vec::new(),
            config,
        })
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        let next_books = {
            let config = slf.config;
            let mut deser = slf.deser.take().expect("Iter without deser");

            let books = &slf.books;

            let next_books = match Self::drive_deserialize(&mut deser, books, config, slf.py()) {
                Ok(bs) => Some(bs),
                Err(err) => {
                    if !err.is_eof() {
                        warn!(target: "betfair_data", "file: {} err: (JSON Parse Error) {}", slf.file_name.to_string_lossy(), err);
                    }

                    None
                }
            };

            slf.deser = Some(deser);

            next_books
        };

        if let Some(next_books) = &next_books {
            slf.books.clone_from(next_books);
        }

        next_books.map(|bs| bs.into_py(slf.py()))
    }

    #[getter]
    fn stream_unique_id(&self) -> Option<u32> {
        self.config.streaming_unique_id
    }
}
