use pyo3::{types::PyTuple, Py, PyAny, PyObject, PyResult, Python};
use std::{lazy::SyncOnceCell, ops::Deref};

use crate::immutable::container::{PyRep, SyncObj};

static DATE_TIME_CLASS: SyncOnceCell<Py<PyAny>> = SyncOnceCell::new();

fn date_time(ts: i64, py: Python) -> PyResult<PyObject> {
    let dtc = DATE_TIME_CLASS.get_or_init(|| {
        let dt_module = py
            .import("datetime")
            .expect("could not import datetime module");
        let dt_class = dt_module
            .getattr("datetime")
            .expect("could not import datetime class");
        Py::from(dt_class)
    });

    let a: [f64; 1] = [ts as f64 / 1000f64];
    let tuple = PyTuple::new(py, a.into_iter());
    dtc.call_method1(py, "utcfromtimestamp", tuple)
}

pub struct DateTimeString {
    str: String,
    ts: i64,
}

impl DateTimeString {
    pub fn new(s: &str) -> Result<Self, chrono::ParseError> {
        let ts = chrono::DateTime::parse_from_rfc3339(s)?;

        Ok(Self {
            str: String::from(s),
            ts: ts.timestamp_millis(),
        })
    }
}

impl PyRep for DateTimeString {
    fn py_rep(&self, py: Python) -> PyObject {
        date_time(self.ts, py).unwrap_or_else(|_| py.None())
    }
}

impl PyRep for Option<DateTimeString> {
    fn py_rep(&self, py: Python) -> PyObject {
        self.as_ref().map_or_else(
            || py.None(),
            |s| date_time(s.ts, py).unwrap_or_else(|_| py.None()),
        )
    }
}

impl Deref for DateTimeString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.str
    }
}

impl PartialEq<str> for DateTimeString {
    fn eq(&self, s: &str) -> bool {
        self.str == s
    }
}

impl PartialEq<DateTimeString> for &str {
    fn eq(&self, s: &DateTimeString) -> bool {
        self == &s.str
    }
}

impl PartialEq<SyncObj<DateTimeString>> for &str {
    fn eq(&self, so: &SyncObj<DateTimeString>) -> bool {
        *self == so.value.as_str()
    }
}




#[derive(Clone, Copy)]
pub struct DateTime(u64);

impl DateTime {
    pub fn new(ts: u64) -> Self {
        Self(ts)
    }
}

impl PyRep for DateTime {
    fn py_rep(&self, py: Python) -> PyObject {
        date_time(self.0 as i64, py).unwrap_or_else(|_| py.None())
    }
}

impl Deref for DateTime {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<u64> for DateTime {
    fn eq(&self, ts: &u64) -> bool {
        self.0 == *ts
    }
}
