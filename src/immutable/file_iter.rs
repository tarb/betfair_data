use pyo3::prelude::*;
use std::path::Path;

use super::market::{Market, MarketsDeser};
use crate::config::Config;
use crate::file_iter::{FileIter, IntoMarketIter, MarketID};
use crate::market_source::{Adapter, MarketSource, SourceConfig, SourceItem};

#[pyclass]
pub struct ImmutAdapter {
    inner: Adapter<File>,
}

impl ImmutAdapter {
    pub fn new(source: Box<dyn MarketSource + Send>) -> Self {
        Self {
            inner: Adapter::new(source),
        }
    }
}

#[pymethods]
impl ImmutAdapter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PyObject> {
        self.inner.next().map(|f| f.into_py(py))
    }
}

#[pyclass(name = "File")]
pub struct File {
    inner: FileIter<Market, ImmutableRep>,
}

#[pymethods]
impl File {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PyObject> {
        self.inner.next(py)
    }

    #[getter]
    fn file_name(&self) -> &Path {
        self.inner.file_name()
    }
}

impl From<(SourceItem, SourceConfig)> for File {
    fn from(s: (SourceItem, SourceConfig)) -> Self {
        Self {
            inner: FileIter::from(s),
        }
    }
}

pub struct ImmutableRep();
impl IntoMarketIter for ImmutableRep {
    type Market = Market;
    type Deser<'a, 'de, 'py> = MarketsDeser<'a, 'py>;

    fn new<'a, 'de, 'py>(
        books: &'a [Py<Self::Market>],
        py: Python<'py>,
        config: Config,
    ) -> Self::Deser<'a, 'de, 'py> {
        MarketsDeser {
            markets: books,
            py,
            config,
        }
    }
}
impl MarketID for Market {
    fn id(&self) -> &str {
        self.market_id.as_str()
    }
}
