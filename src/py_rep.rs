use pyo3::{
    prelude::*,
    types::{PyList, PyUnicode},
};
use staticvec::StaticString;
use std::path::PathBuf;

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

impl PyRep for str {
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

impl<const N: usize> PyRep for StaticString<N> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyUnicode::new(py, self.as_str()).into_py(py)
    }
}

impl PyRep for Vec<String> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}
