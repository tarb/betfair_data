use pyo3::{types::PyTuple, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::{lazy::SyncOnceCell, ops::Deref};

use crate::{py_rep::PyRep, strings::FixedSizeString};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct DateTimeString {
    str: FixedSizeString<24>,
    ts: i64,
}

impl DateTimeString {
    pub fn new(s: &str) -> Result<Self, chrono::ParseError> {
        let ts = chrono::DateTime::parse_from_rfc3339(s)?;

        Ok(Self {
            str: FixedSizeString::try_from(s).unwrap(),
            ts: ts.timestamp_millis(),
        })
    }

    pub fn as_fs_str(&self) -> FixedSizeString<24> {
        self.str
    }
}

impl TryFrom<FixedSizeString<24>> for DateTimeString {
    type Error = chrono::ParseError;

    fn try_from(str: FixedSizeString<24>) -> Result<Self, Self::Error> {
        let ts = chrono::DateTime::parse_from_rfc3339(&str)?.timestamp_millis();
        Ok(Self { str, ts })
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

impl AsRef<str> for DateTimeString {
    fn as_ref(&self) -> &str {
        self.str.as_str()
    }
}

impl Deref for DateTimeString {
    type Target = FixedSizeString<24>;

    fn deref(&self) -> &Self::Target {
        &self.str
    }
}

impl PartialEq<str> for DateTimeString {
    fn eq(&self, s: &str) -> bool {
        s == self.str
    }
}

impl PartialEq<DateTimeString> for &str {
    fn eq(&self, s: &DateTimeString) -> bool {
        self == &s.str
    }
}

impl PartialEq<DateTimeString> for FixedSizeString<24> {
    fn eq(&self, s: &DateTimeString) -> bool {
        *self == s.str
    }
}

#[derive(Clone, Copy, Debug, Default)]
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

impl ToPyObject for DateTime {
    fn to_object(&self, py: Python) -> PyObject {
        date_time(self.0 as i64, py).unwrap_or_else(|_| py.None())
    }
}

impl IntoPy<PyObject> for DateTime {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}
