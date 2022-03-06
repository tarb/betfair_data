use pyo3::{prelude::*};

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};

use std::fmt;

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct FloatStr(pub f64);

impl From<FloatStr> for f64 {
    fn from(v: FloatStr) -> f64 {
        v.0
    }
}

impl<'de> Deserialize<'de> for FloatStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct F64Visitor;
        impl<'a> Visitor<'a> for F64Visitor {
            type Value = FloatStr;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a f64 or a string containing digits, NaN or Infinity"
                )
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(FloatStr(v as f64))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(FloatStr(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(FloatStr(v))
            }

            fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
                match s {
                    "NaN" => Ok(FloatStr(f64::NAN)),
                    "Infinity" => Ok(FloatStr(f64::INFINITY)),
                    _ => s
                        .parse()
                        .map_err(|_| Error::custom("invalid PriceSize string value"))
                        .map(FloatStr),
                }
            }
        }

        deserializer.deserialize_any(F64Visitor)
    }
}

impl IntoPy<PyObject> for FloatStr {
    fn into_py(self, py: Python) -> PyObject {
        if self.0.is_nan() {
            "NaN".into_py(py)
        } else if self.0.is_infinite() {
            "Infinity".into_py(py)
        } else {
            self.0.into_py(py)
        }
    }
}