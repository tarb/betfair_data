use log::warn;
use pyo3::{exceptions, prelude::*};
use serde::de::DeserializeSeed;
use std::path::PathBuf;

use super::market_book::{MarketBook, MarketBooksDeser};
use crate::deser::DeserializerWithData;
use crate::market_source::{Adapter, MarketSource, SourceConfig, SourceItem};

#[pyclass]
pub struct BflwAdapter {
    inner: Adapter<BflwFile>,
}

impl BflwAdapter {
    pub fn new(source: Box<dyn MarketSource + Send>) -> Self {
        Self {
            inner: Adapter::new(source),
        }
    }
}

#[pymethods]
impl BflwAdapter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PyObject> {
        self.inner.next().map(|f| f.into_py(py))
    }
}

#[pyclass(name = "File")]
pub struct BflwFile {
    #[pyo3(get)]
    file_name: PathBuf,
    config: SourceConfig,
    deser: Option<DeserializerWithData>,
    books: Vec<Py<MarketBook>>,
}

impl BflwFile {
    fn drive_deserialize(
        deser: &mut DeserializerWithData,
        books: &[Py<MarketBook>],
        config: SourceConfig,
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

impl From<(SourceItem, SourceConfig)> for BflwFile {
    fn from(s: (SourceItem, SourceConfig)) -> Self {
        let (item, config) = s;

        Self {
            file_name: item.file,
            deser: Some(item.deser),
            books: Vec::new(),
            config,
        }
    }
}

#[pymethods]
impl BflwFile {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    fn __new__(file: PathBuf, bytes: &[u8], cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
        };

        let deser = DeserializerWithData::build(bytes.to_owned())
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        Ok(Self {
            file_name: file,
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
}
