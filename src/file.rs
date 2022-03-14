use pyo3::{prelude::*};
use std::path::{Path, PathBuf};

use crate::file_iter::FileIter;
use crate::immutable::file_iter::ImmutableRep;
use crate::immutable::market::Market;
use crate::mutable::file_iter::MutableRep;
use crate::mutable::market::MarketMut;

enum FileType {
    Mutable(FileIter<MarketMut, MutableRep>),
    Immutable(FileIter<Market, ImmutableRep>),
}

#[pyclass(name = "File")]
pub struct File {
    inner: FileType,
}

#[pymethods]
impl File {
    #[new]
    #[args(cumulative_runner_tv = "true")]
    #[args(mutable = "false")]
    fn __new__(
        file: PathBuf,
        bytes: &[u8],
        cumulative_runner_tv: bool,
        mutable: bool,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: match mutable {
                true => FileType::Mutable(FileIter::new(file, bytes, cumulative_runner_tv)?),
                false => FileType::Immutable(FileIter::new(file, bytes, cumulative_runner_tv)?),
            },
        })
    }

    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<PyObject> {
        match &mut self.inner {
            FileType::Immutable(inner) => inner.next(py),
            FileType::Mutable(inner) => inner.next(py),
        }
    }

    #[getter]
    fn file_name(&self) -> &Path {
        match &self.inner {
            FileType::Immutable(inner) => inner.file_name(),
            FileType::Mutable(inner) => inner.file_name(),
        }
    }
}
