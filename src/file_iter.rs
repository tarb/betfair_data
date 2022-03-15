use log::warn;
use pyo3::{exceptions, prelude::*};
use serde::de::DeserializeSeed;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::deser::DeserializerWithData;
use crate::immutable::container::SyncObj;
use crate::market_source::{SourceConfig, SourceItem};

pub trait IntoMarketIter {
    type Market: pyo3::PyClass + MarketID;
    type Deser<'a, 'de, 'py>: DeserializeSeed<'de, Value = VecDeque<Py<Self::Market>>>;

    fn new<'a, 'de, 'py>(
        books: &'a [Py<Self::Market>],
        py: Python<'py>,
        config: Config,
    ) -> Self::Deser<'a, 'de, 'py>;
}

pub trait MarketID {
    fn id(&self) -> &str;
}

pub struct FileIter<T: pyo3::PyClass + MarketID, I: IntoMarketIter<Market = T>> {
    file_name: SyncObj<PathBuf>,
    config: Config,
    deser: Option<DeserializerWithData>,
    books: Vec<Py<T>>,
    iter_stack: VecDeque<Py<T>>,
    pd: PhantomData<I>,
}

impl<T: pyo3::PyClass + MarketID, I: IntoMarketIter<Market = T>> From<(SourceItem, SourceConfig)>
    for FileIter<T, I>
{
    fn from(s: (SourceItem, SourceConfig)) -> Self {
        let (item, config) = s;

        let config = Config {
            cumulative_runner_tv: config.cumulative_runner_tv,
        };

        Self {
            file_name: SyncObj::new(item.file),
            deser: Some(item.deser),
            books: Vec::new(),
            iter_stack: VecDeque::new(),
            config,
            pd: Default::default(),
        }
    }
}

impl<T: pyo3::PyClass + MarketID, I: IntoMarketIter<Market = T>> FileIter<T, I> {
    pub fn new(file: PathBuf, bytes: &[u8], cumulative_runner_tv: bool) -> PyResult<Self> {
        let config = Config {
            cumulative_runner_tv,
        };

        let deser = DeserializerWithData::build(bytes.to_owned())
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        Ok(Self {
            file_name: SyncObj::new(file),
            deser: Some(deser),
            books: Vec::new(),
            iter_stack: VecDeque::new(),
            config,
            pd: Default::default(),
        })
    }

    pub fn file_name(&self) -> &Path {
        &*self.file_name
    }

    pub fn next(&mut self, py: Python) -> Option<PyObject> {
        if let Some(m) = self.iter_stack.pop_front() {
            let index = {
                let market = m.borrow(py);
                self.books
                    .iter()
                    .position(|m2| market.id() == (*m2).borrow(py).id())
            };

            let mc = m.clone_ref(py);
            match index {
                Some(i) => self.books[i] = mc,
                None => self.books.push(mc),
            }

            Some(m.into_py(py))
        } else {
            let next_books = {
                let mut deser = self.deser.take().expect("Iter without deser");

                let books = self.books.as_slice();
                let next_books = deser.with_dependent_mut(|_, deser| {
                    I::new(books, py, self.config).deserialize(&mut deser.0)
                });

                let next_books = match next_books {
                    Ok(bs) => Some(bs),
                    Err(err) => {
                        if !err.is_eof() {
                            warn!(target: "betfair_data", "file: {} err: (JSON Parse Error) {}", self.file_name.to_string_lossy(), err);
                        }

                        None
                    }
                };

                self.deser = Some(deser);

                next_books
            };

            next_books.and_then(|mut next_books| {
                next_books.pop_front().map(|m| {
                    let index = {
                        let market = &(*m.borrow(py));
                        self.books
                            .iter()
                            .position(|m2| market.id() == (*m2).borrow(py).id())
                    };

                    let mc = m.clone_ref(py);
                    match index {
                        Some(i) => self.books[i] = mc,
                        None => self.books.push(mc),
                    }

                    self.iter_stack = next_books;
                    m.into_py(py)
                })
            })
        }
    }
}
