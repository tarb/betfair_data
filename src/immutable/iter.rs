use log::warn;
use pyo3::{exceptions, prelude::*, PyIterProtocol};
use serde::de::DeserializeSeed;
use std::collections::VecDeque;
use std::path::PathBuf;

use super::container::SyncObj;
use super::market::{PyMarket, PyMarketsDeser};
use crate::deser::DeserializerWithData;
use crate::market_source::{SourceConfig, SourceItem};

#[pyclass(name = "ImmutIter")]
pub struct ImmutIter {
    #[pyo3(get)]
    file_name: SyncObj<PathBuf>,
    config: SourceConfig,
    deser: Option<DeserializerWithData>,
    books: Vec<Py<PyMarket>>,

    iter_stack: VecDeque<Py<PyMarket>>,
}

impl ImmutIter {
    pub fn new_object(item: SourceItem, config: SourceConfig, py: Python) -> PyObject {
        ImmutIter {
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
        books: &[Py<PyMarket>],
        config: SourceConfig,
        py: Python,
    ) -> Result<VecDeque<Py<PyMarket>>, serde_json::Error> {
        deser.with_dependent_mut(|_, deser| {
            PyMarketsDeser {
                markets: books,
                py,
                config,
            }
            .deserialize(&mut deser.0)
        })
    }
}

#[pymethods]
impl ImmutIter {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    fn __new__(file: PathBuf, bytes: &[u8], cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
        };

        let deser = DeserializerWithData::build(bytes.to_owned())
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        Ok(ImmutIter {
            file_name: SyncObj::new(file),
            deser: Some(deser),
            books: Vec::new(),
            iter_stack: VecDeque::new(),
            config,
        })
    }
}

#[pyproto]
impl<'p> PyIterProtocol for ImmutIter {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        if let Some(m) = slf.iter_stack.pop_front() {
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

            // update the books for this files
            if let Some(mut next_books) = next_books {
                let mut books = slf
                    .books
                    .iter()
                    .map(|b| b.clone_ref(slf.py()))
                    .collect::<Vec<_>>();
                next_books.iter().for_each(|m1| {
                    let mb1 = m1.borrow(slf.py());
                    let id1: &str = mb1.market_id.as_ref();

                    let replace = books
                        .iter_mut()
                        .position(|m2| id1 == (*m2).borrow(slf.py()).market_id);

                    match replace {
                        Some(i) => books[i] = m1.clone_ref(slf.py()),
                        None => books.push(m1.clone_ref(slf.py())),
                    }
                });

                // update books cache
                slf.books = books;

                // pop from front to return and then add rest to iter stack
                let next_market = next_books.pop_front();
                slf.iter_stack = next_books;

                next_market.map(|m| m.into_py(slf.py()))
            } else {
                None
            }
        }
    }
}
