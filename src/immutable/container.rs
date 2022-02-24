use pyo3::{
    prelude::*,
    types::{PyList, PyUnicode},
};
use std::lazy::SyncOnceCell;
use std::sync::Arc;

use crate::{ids::MarketID, price_size::PriceSize};

pub trait PyRep {
    fn py_rep(&self, py: Python) -> PyObject;
}

impl PyRep for Vec<PriceSize> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

impl PyRep for String {
    fn py_rep(&self, py: Python) -> PyObject {
        PyUnicode::new(py, self).into_py(py)
    }
}

impl PyRep for MarketID {
    fn py_rep(&self, py: Python) -> PyObject {
        PyUnicode::new(py, self).into_py(py)
    }
}

impl PyRep for Vec<String> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

#[derive(Debug)]
pub struct SyncObj<T> {
    pub value: Arc<T>,
    py: SyncOnceCell<PyObject>,
}

impl<T: PyRep> SyncObj<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(value),
            py: SyncOnceCell::new(),
        }
    }
}

impl<T: PyRep> Clone for SyncObj<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            py: self.py.clone(),
        }
    }
}

impl<T: PyRep> ToPyObject for SyncObj<T> {
    fn to_object(&self, py: Python) -> PyObject {
        self.py.get_or_init(|| self.value.py_rep(py)).clone_ref(py)
    }
}

impl<T: PyRep> IntoPy<PyObject> for SyncObj<T> {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

impl<T: PyRep + Default> Default for SyncObj<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl PartialEq<SyncObj<String>> for &str {
    fn eq(&self, so: &SyncObj<String>) -> bool {
        *self == so.value.as_str()
    }
}
