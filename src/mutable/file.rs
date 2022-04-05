use pyo3::prelude::*;
use std::path::Path;

use crate::config::Config;
use crate::file_iter::{FileIter, IntoMarketIter, MarketID};
use crate::market_source::SourceItem;
use crate::mutable::market::{MarketMut, MarketMutDeser};

#[pyclass(name = "File")]
pub struct File {
    inner: FileIter<MarketMut, MutableRep>,
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

pub struct MutableRep();
impl IntoMarketIter for MutableRep {
    type Market = MarketMut;
    type Deser<'a, 'de, 'py> = MarketMutDeser<'a, 'py>;

    fn new<'a, 'de, 'py>(
        books: &'a [Py<Self::Market>],
        py: Python<'py>,
        config: Config,
    ) -> Self::Deser<'a, 'de, 'py> {
        MarketMutDeser {
            markets: books,
            py,
            config,
        }
    }
}
impl MarketID for MarketMut {
    fn id(&self) -> &str {
        self.market_id.as_str()
    }
}
