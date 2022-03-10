use pyo3::{
    prelude::*,
    types::{PyList, PyUnicode},
};

use std::sync::Arc;
use std::{lazy::OnceCell, path::PathBuf};

use super::datetime::DateTimeString;
use crate::{price_size::PriceSize, strings::FixedSizeString};

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

impl PyRep for PathBuf {
    fn py_rep(&self, py: Python) -> PyObject {
        PyUnicode::new(py, self.to_string_lossy().as_ref()).into_py(py)
    }
}

impl<const N: usize> PyRep for FixedSizeString<N> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyUnicode::new(py, self.as_str()).into_py(py)
    }
}

impl PyRep for Vec<String> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

#[derive(Debug)]
pub struct SyncObj<T> {
    value: T,
    py: OnceCell<PyObject>,
}

unsafe impl<T: Sync> Sync for SyncObj<T> {}

impl<T> std::ops::Deref for SyncObj<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> SyncObj<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            py: OnceCell::new(),
        }
    }
}

impl<T: Clone> Clone for SyncObj<T> {
    default fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            py: self.py.clone(),
        }
    }
}

impl<T: Clone + Copy> Clone for SyncObj<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            py: self.py.clone(),
        }
    }
}

impl<T: PyRep> ToPyObject for SyncObj<T> {
    fn to_object(&self, py: Python) -> PyObject {
        self.py.get_or_init(|| self.value.py_rep(py)).clone_ref(py)
    }
}

impl<T: PyRep> ToPyObject for SyncObj<Arc<T>> {
    fn to_object(&self, py: Python) -> PyObject {
        self.py.get_or_init(|| self.value.py_rep(py)).clone_ref(py)
    }
}

impl<T: PyRep> IntoPy<PyObject> for SyncObj<T> {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

impl<T: PyRep> IntoPy<PyObject> for SyncObj<Arc<T>> {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

impl<T: PyRep + Default> Default for SyncObj<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            py: OnceCell::new(),
        }
    }
}

impl<T: PyRep + Default> Default for SyncObj<Arc<T>> {
    fn default() -> Self {
        Self {
            value: Arc::new(Default::default()),
            py: OnceCell::new(),
        }
    }
}

impl<T: AsRef<str>> AsRef<str> for SyncObj<T> {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

impl PartialEq<SyncObj<String>> for &str {
    fn eq(&self, so: &SyncObj<String>) -> bool {
        *self == so.value
    }
}

impl PartialEq<SyncObj<Arc<String>>> for &str {
    fn eq(&self, so: &SyncObj<Arc<String>>) -> bool {
        *self == so.value.as_ref()
    }
}

impl<const N: usize> PartialEq<SyncObj<FixedSizeString<N>>> for &str {
    fn eq(&self, so: &SyncObj<FixedSizeString<N>>) -> bool {
        self == &so.value
    }
}

impl PartialEq<SyncObj<DateTimeString>> for &str {
    fn eq(&self, so: &SyncObj<DateTimeString>) -> bool {
        *self == so.value.as_ref()
    }
}
