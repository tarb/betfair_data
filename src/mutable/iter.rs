use log::warn;
use pyo3::{exceptions, prelude::*, PyIterProtocol};
use serde::de::DeserializeSeed;
use std::collections::VecDeque;
use std::path::PathBuf;

use crate::config::Config;
use crate::immutable::container::SyncObj;
use super::market::{PyMarketMut, PyMarketToken};
use crate::deser::DeserializerWithData;
use crate::market_source::{SourceConfig, SourceItem};

#[pyclass(name = "MutIter")]
pub struct MutIter {
    #[pyo3(get)]
    file_name: SyncObj<PathBuf>,
    config: SourceConfig,
    deser: Option<DeserializerWithData>,
    books: Vec<Py<PyMarketMut>>,

    iter_stack: VecDeque<Py<PyMarketMut>>,
}

impl MutIter {
    pub fn new_object(item: SourceItem, config: SourceConfig, py: Python) -> PyObject {
        MutIter {
            file_name: SyncObj::new(item.file),
            deser: Some(item.deser),
            books: Vec::new(),
            iter_stack: VecDeque::new(),
            config,
        }
        .into_py(py)
    }

    fn drive_deserialize(
        deser: &mut DeserializerWithData,
        books: &[Py<PyMarketMut>],
        config: SourceConfig,
        py: Python,
    ) -> Result<VecDeque<Py<PyMarketMut>>, serde_json::Error> {
        deser.with_dependent_mut(|_, deser| {
            let config = Config{ cumulative_runner_tv: config.cumulative_runner_tv};
            
            PyMarketToken {
                markets: books,
                py,
                config,
            }
            .deserialize(&mut deser.0)
        })
    }
}

#[pymethods]
impl MutIter {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    fn __new__(file: PathBuf, bytes: &[u8], cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
        };

        let deser = DeserializerWithData::build(bytes.to_owned())
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        Ok(MutIter {
            file_name: SyncObj::new(file),
            deser: Some(deser),
            books: Vec::new(),
            iter_stack: VecDeque::new(),
            config,
        })
    }
}

#[pyproto]
impl<'p> PyIterProtocol for MutIter {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        if let Some(m) = slf.iter_stack.pop_front() {
            let index = {
                let market = m.borrow(slf.py());
                slf.books
                    .iter()
                    .position(|m2| market.market_id.as_str() == (*m2).borrow(slf.py()).market_id.as_str())
            };

            let mc = m.clone_ref(slf.py());
            match index {
                Some(i) => slf.books[i] = mc,
                None => slf.books.push(mc),
            }

            Some(m.into_py(slf.py()))
        } else {
            let next_books = {
                let config = slf.config;
                let mut deser = slf.deser.take().expect("Iter without deser");

                let books = &slf.books;

                let next_books = match Self::drive_deserialize(&mut deser, books, config, slf.py())
                {
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

            next_books.and_then(|mut next_books| {
                next_books.pop_front().map(|m| {
                    let index = {
                        let market = m.borrow(slf.py());
                        slf.books
                            .iter()
                            .position(|m2| market.market_id.as_str() == (*m2).borrow(slf.py()).market_id.as_str())
                    };
        
                    let mc = m.clone_ref(slf.py());
                    match index {
                        Some(i) => slf.books[i] = mc,
                        None => slf.books.push(mc),
                    }
        
                    slf.iter_stack = next_books;
                    m.into_py(slf.py())
                })
            })
        }
    }
}