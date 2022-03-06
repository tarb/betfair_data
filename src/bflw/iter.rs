use log::warn;
use pyo3::{exceptions, prelude::*, PyIterProtocol};
use serde::de::DeserializeSeed;
use std::path::PathBuf;

use super::market_book::{MarketBook, MarketBooksDeser};
use crate::deser::DeserializerWithData;
use crate::market_source::{SourceConfig, SourceItem};

#[pyclass(name = "BflwIter")]
pub struct BflwIter {
    #[pyo3(get)]
    file_name: PathBuf,
    config: SourceConfig,
    deser: Option<DeserializerWithData>,
    books: Vec<Py<MarketBook>>,
}

impl BflwIter {
    pub fn new_object(item: SourceItem, config: SourceConfig, py: Python) -> PyObject {
        BflwIter {
            file_name: item.file,
            deser: Some(item.deser),
            books: Vec::new(),
            config,
        }
        .into_py(py)
    }

    fn drive_deserialize(
        deser: &mut DeserializerWithData,
        books: &[Py<MarketBook>],
        config: SourceConfig,
        py: Python,
    ) -> Result<Vec<Py<MarketBook>>, serde_json::Error> {
        deser.with_dependent_mut(|_, deser| {
            MarketBooksDeser{markets: books, py, config}.deserialize(&mut deser.0)
        })
    }
}

#[pymethods]
impl BflwIter {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    fn __new__(file: PathBuf, bytes: &[u8], cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
        };

        let deser = DeserializerWithData::build(bytes.to_owned())
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        Ok(BflwIter {
            file_name: file,
            deser: Some(deser),
            books: Vec::new(),
            config,
        })
    }

}

#[pyproto]
impl<'p> PyIterProtocol for BflwIter {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
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

        // update the books for this files
        if let Some(next_books) = &next_books {
            let py = slf.py();

            let mut books = slf
                .books
                .iter()
                .map(|b| b.clone_ref(py))
                .collect::<Vec<_>>();
            next_books.iter().for_each(|m1| {
                let mb1 = m1.borrow(py);
                let id1: &str = mb1.market_id.as_ref();

                let replace = books
                    .iter_mut()
                    .position(|m2| id1 == (*m2).borrow(py).market_id);

                match replace {
                    Some(i) => books[i] = m1.clone_ref(py),
                    None => books.push(m1.clone_ref(py)),
                }
            });

            slf.books = books;
        }

        next_books.map(|bs| bs.into_py(slf.py()))
    }
}
