use pyo3::prelude::*;
use std::lazy::OnceCell;
use std::sync::Arc;

use crate::py_rep::PyRep;

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

impl<T: PyRep + ?Sized> ToPyObject for SyncObj<Arc<T>> {
    fn to_object(&self, py: Python) -> PyObject {
        self.py.get_or_init(|| self.value.py_rep(py)).clone_ref(py)
    }
}

impl<T: PyRep> IntoPy<PyObject> for SyncObj<T> {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

impl<T: PyRep + ?Sized> IntoPy<PyObject> for SyncObj<Arc<T>> {
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

impl<T: PyRep + Default + ?Sized> Default for SyncObj<Arc<T>> {
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

impl<T: AsRef<str>> PartialEq<SyncObj<T>> for &str {
    fn eq(&self, so: &SyncObj<T>) -> bool {
        *self == so.value.as_ref()
    }
}
