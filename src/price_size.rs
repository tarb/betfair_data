use pyo3::prelude::*;
use serde::{
    de::{Error, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::cmp::PartialEq;

use std::fmt;

/**
 * PriceSize
 * Eq PartialEq, when price and size are equal
 * Ord, Ordered by price field
 */
#[pyclass]
#[derive(Debug, Copy, Clone)]
pub struct PriceSize {
    #[pyo3(get)]
    pub price: f64,
    #[pyo3(get)]
    pub size: f64,
}

impl PriceSize {
    pub fn new(price: f64, size: f64) -> Self {
        Self { price, size }
    }
}

impl Eq for PriceSize {}
impl PartialEq for PriceSize {
    fn eq(&self, other: &PriceSize) -> bool {
        self.price == other.price && self.size == other.size
    }
}

impl ToPyObject for PriceSize {
    fn to_object(&self, py: Python) -> PyObject {
        self.into_py(py)
    }
}

#[pymethods]
impl PriceSize {
    #[new]
    fn py_constructor(price: f64, size: f64) -> Self {
        Self { price, size }
    }

    fn __str__(&self) -> String {
        format!("[{:.2},{:.2}]", self.price, self.size)
    }

    fn __repr__(&self) -> String {
        format!("<PriceSize [{:.2},{:.2}]>", self.price, self.size)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct F64OrStr(f64);

impl From<F64OrStr> for f64 {
    fn from(v: F64OrStr) -> f64 {
        v.0
    }
}

impl std::ops::Deref for F64OrStr {
    type Target = f64;

    #[inline]
    fn deref(&self) -> &f64 {
        &self.0
    }
}

impl<'de> Deserialize<'de> for F64OrStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct F64Visitor;
        impl<'a> Visitor<'a> for F64Visitor {
            type Value = F64OrStr;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a f64 or a string containing digits, NaN or Infinity"
                )
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
                Ok(F64OrStr(v as f64))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
                Ok(F64OrStr(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
                Ok(F64OrStr(v))
            }

            fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
                match s {
                    "NaN" => Ok(F64OrStr(f64::NAN)),
                    "Infinity" => Ok(F64OrStr(f64::INFINITY)),
                    _ => s
                        .parse()
                        .map_err(|_| Error::custom("invalid PriceSize string value"))
                        .map(F64OrStr),
                }
            }
        }

        deserializer.deserialize_any(F64Visitor)
    }
}

impl<'de> Deserialize<'de> for PriceSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PriceSizeVisitor;
        impl<'de> Visitor<'de> for PriceSizeVisitor {
            type Value = PriceSize;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tuple in the form [price, size], eg [2.04,234.1] or a object in the form {\"price\":2.04,\"size\":234.1}")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let price: F64OrStr = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let size: F64OrStr = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(1, &self))?;

                Ok(PriceSize::new(price.0, size.0))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                #[derive(Debug, Deserialize)]
                #[serde(field_identifier, rename_all = "camelCase")]
                enum Field {
                    Price,
                    Size,
                }

                let mut price: Option<F64OrStr> = None;
                let mut size: Option<F64OrStr> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Price => {
                            if price.is_some() {
                                return Err(Error::duplicate_field("price"));
                            }
                            price = Some(map.next_value()?);
                        }
                        Field::Size => {
                            if size.is_some() {
                                return Err(Error::duplicate_field("size"));
                            }
                            size = Some(map.next_value()?);
                        }
                    }
                }

                let price = price.ok_or_else(|| Error::missing_field(FIELDS[0]))?;
                let size = size.ok_or_else(|| Error::missing_field(FIELDS[1]))?;
                Ok(PriceSize::new(price.0, size.0))
            }
        }

        const FIELDS: &[&str] = &["price", "size"];
        deserializer.deserialize_struct("PriceSize", FIELDS, PriceSizeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricesize_deserialize() {
        let raw = r#"["Infinity", 5]"#;
        let pv: PriceSize = serde_json::from_str(raw).expect("failed to deserialize");
        assert_eq!(pv, PriceSize::new(f64::INFINITY, 5.0));

        let raw = r#"[1.2, "Infinity"]"#;
        let pv: PriceSize = serde_json::from_str(raw).expect("failed to deserialize");
        assert_eq!(pv, PriceSize::new(1.2, f64::INFINITY));

        let raw = r#"{"price": 2.0, "size": 1.0}"#;
        let pv: PriceSize = serde_json::from_str(raw).expect("failed to deserialize");
        assert_eq!(pv, PriceSize::new(2.0, 1.0));

        let raw = r#"{"price": "NaN", "size": "NaN"}"#;
        let pv: PriceSize = serde_json::from_str(raw).expect("failed to deserialize");
        assert!(pv.price.is_nan() && pv.size.is_nan());

        let raw = r#"{"price": 2.0, "size": "NaN"}"#;
        let pv: PriceSize = serde_json::from_str(raw).expect("failed to serialize");
        assert!(pv.price == 2.0 && pv.size.is_nan());
    }
}
