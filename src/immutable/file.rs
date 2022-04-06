use pyo3::prelude::*;
use std::path::Path;

use super::market::{Market, MarketsDeser};
use crate::config::Config;
use crate::file_iter::{FileIter, IntoMarketIter, MarketID};
use crate::market_source::SourceItem;

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

impl From<(SourceItem, Config)> for File {
    fn from(s: (SourceItem, Config)) -> Self {
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
