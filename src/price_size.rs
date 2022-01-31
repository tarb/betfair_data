use std::fmt;
use std::cmp::Ordering;
use pyo3::{prelude::*, PyObjectProtocol}; 
use serde::{ 
    Deserialize, Deserializer, 
    de::{ SeqAccess, MapAccess, Visitor, Error, DeserializeSeed },
};

const MIN_VEC_CAP: usize = 20;

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

#[pyproto]
impl PyObjectProtocol for PriceSize {
    fn __str__(&self) -> String {
        format!("[{:.2},{:.2}]", self.price, self.size)
    }

    fn __repr__(&self) -> String {
        format!("<PriceSize [{:.2},{:.2}]>", self.price, self.size)
    }
}

#[pymethods]
impl PriceSize {
    #[new]
    fn py_constructor(price: f64, size: f64) -> Self {
        Self { price, size }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct F64OrStr(f64);
impl Into<f64> for F64OrStr {
    fn into(self) -> f64 {
        self.0
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
                write!(formatter, "a f64 or a string containing digits, NaN or Infinity")
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
                    _ => s.parse().map_err(|_| Error::custom("invalid PriceSize string value")).map(|v| F64OrStr(v)),
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
                let price: F64OrStr = seq.next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let size: F64OrStr = seq.next_element()?
                    .ok_or_else(|| Error::invalid_length(1, &self))?;
                
                Ok(PriceSize::new(price.0, size.0))
            }
        
            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                #[derive(Debug,Deserialize)]
                #[serde(field_identifier, rename_all = "camelCase")]
                enum Field { Price, Size }

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
        
        const FIELDS: &'static [&'static str] = &["price", "size"];
        deserializer.deserialize_struct("PriceSize", FIELDS, PriceSizeVisitor)
    }
}

pub struct PriceSizeBackLadder<'a>(pub &'a mut Vec<PriceSize>);
impl<'de, 'a> DeserializeSeed<'de> for PriceSizeBackLadder<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct PSVisitor<'a>(&'a mut Vec<PriceSize>);
        impl<'de, 'a> Visitor<'de> for PSVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("An array of PriceSize values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>, 
            {
                // grow an empty vec
                if self.0.capacity() == 0 {
                    match seq.size_hint() {
                        Some(s) => self.0.reserve_exact(std::cmp::max(MIN_VEC_CAP, s + 2)),
                        None => self.0.reserve_exact(MIN_VEC_CAP),
                    }
                }

                while let Some(ps1) = seq.next_element::<PriceSize>()? {
                    let cmp_fn =  |ps2: &PriceSize| {
                        if ps1.price < ps2.price {
                            Ordering::Greater
                        } else if ps1.price > ps2.price {
                            Ordering::Less
                        } else {
                            Ordering::Equal
                        }
                    };

                    if ps1.size == 0.0 {
                        // removing price
                        match self.0.binary_search_by(cmp_fn) {
                            Ok(index) => { self.0.remove(index); },
                            Err(_err) => {},
                        }
                    } else {
                        match self.0.binary_search_by(cmp_fn) {
                            // updating price
                            Ok(index) => self.0.get_mut(index).unwrap().size = ps1.size,
                            // inserting price
                            Err(index) => self.0.insert(index, ps1),
                        }
                    }
                }

                Ok(())
            }
        }

        Ok(deserializer.deserialize_seq(PSVisitor(self.0))?)
    }
}


pub struct PriceSizeLayLadder<'a>(pub &'a mut Vec<PriceSize>);
impl<'de, 'a> DeserializeSeed<'de> for PriceSizeLayLadder<'a> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct PSVisitor<'a>(&'a mut Vec<PriceSize>);
        impl<'de, 'a> Visitor<'de> for PSVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("An array of PriceSize values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>, 
            {
                // grow an empty vec
                if self.0.capacity() == 0 {
                    match seq.size_hint() {
                        Some(s) => self.0.reserve_exact(std::cmp::max(MIN_VEC_CAP, s + 2)),
                        None => self.0.reserve_exact(MIN_VEC_CAP),
                    }
                }

                while let Some(ps1) = seq.next_element::<PriceSize>()? {
                    let cmp_fn =  |ps2: &PriceSize| {
                        if ps1.price < ps2.price {
                            Ordering::Less
                        } else if ps1.price > ps2.price {
                            Ordering::Greater
                        } else {
                            Ordering::Equal
                        }
                    };

                    if ps1.size == 0.0 {
                        // removing price
                        match self.0.binary_search_by(cmp_fn) {
                            Ok(index) => { self.0.remove(index); },
                            Err(_err) => {},
                        }
                    } else {
                        match self.0.binary_search_by(cmp_fn) {
                            // updating price
                            Ok(index) => self.0.get_mut(index).unwrap().size = ps1.size,
                            // inserting price
                            Err(index) => self.0.insert(index, ps1),
                        }
                    }
                }

                Ok(())
            }
        }

        Ok(deserializer.deserialize_seq(PSVisitor(self.0))?)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::PartialEq;

    impl Eq for PriceSize {}
    impl PartialEq for PriceSize {
        fn eq(&self, other: &PriceSize) -> bool {
            self.price == other.price && self.size == other.size
        }
    }

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

    #[test]
    fn test_back_ladder_deserialize() {
        let raw = r#"
            [[5, 5],[3.0, 5],[2, 5],[6, 5],[4, 5]]
            [[1.5, 4],[3.5, 4],[2.5, 4.0],[1.25, 4],[7, 4]]
            [[7, 0],[1.5, 0.0],[5, 0],[1.25, 0]]
            [[2, 1],[2.5, 0],[3, 1],[3.5, 0],[4, 1],[5.0,1.0]]
            [[2, 0],[3, 0],[4.0, 0],[5, 0],[6.0, 0]]
            [[1.6, 1],[1.1, 6],[1.4, 3],[1.3, 4],[1.5, 2],[1.2, 5]]
            [[2.2, 2],[1.9, 2],[1.7, 2],[2.1, 2],[1.8, 2],[2.0, 2]]
            [[3.0, 3],[2.3, 3],[2.9, 3],[2.4, 3],[2.8, 3],[2.5, 3],[2.7, 3],[2.6, 3],[3.1, 3]]
        "#;
        let mut deser = serde_json::Deserializer::from_str(raw);
        
        let mut ps: Vec<PriceSize> = Vec::new();

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps.capacity(), MIN_VEC_CAP);        
        assert_eq!(ps, vec![
            PriceSize::new(2.0,5.0),
            PriceSize::new(3.0,5.0),
            PriceSize::new(4.0,5.0),
            PriceSize::new(5.0,5.0),
            PriceSize::new(6.0,5.0),
        ]);

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(1.25,4.0),
            PriceSize::new(1.5,4.0),
            PriceSize::new(2.0,5.0),
            PriceSize::new(2.5,4.0),
            PriceSize::new(3.0,5.0),
            PriceSize::new(3.5,4.0),
            PriceSize::new(4.0,5.0),
            PriceSize::new(5.0,5.0),
            PriceSize::new(6.0,5.0),
            PriceSize::new(7.0,4.0),
        ]);

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(2.0,5.0),
            PriceSize::new(2.5,4.0),
            PriceSize::new(3.0,5.0),
            PriceSize::new(3.5,4.0),
            PriceSize::new(4.0,5.0),
            PriceSize::new(6.0,5.0),
        ]);

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(2.0,1.0),
            PriceSize::new(3.0,1.0),
            PriceSize::new(4.0,1.0),
            PriceSize::new(5.0,1.0),
            PriceSize::new(6.0,5.0),
        ]);

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert!(ps.is_empty());
        
        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(1.1,6.0),
            PriceSize::new(1.2,5.0),
            PriceSize::new(1.3,4.0),
            PriceSize::new(1.4,3.0),
            PriceSize::new(1.5,2.0),
            PriceSize::new(1.6,1.0),
        ]);

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert!(ps.capacity() == MIN_VEC_CAP);
        assert_eq!(ps, vec![
            PriceSize::new(1.1,6.0),
            PriceSize::new(1.2,5.0),
            PriceSize::new(1.3,4.0),
            PriceSize::new(1.4,3.0),
            PriceSize::new(1.5,2.0),
            PriceSize::new(1.6,1.0),
            PriceSize::new(1.7,2.0),
            PriceSize::new(1.8,2.0),
            PriceSize::new(1.9,2.0),
            PriceSize::new(2.0,2.0),
            PriceSize::new(2.1,2.0),
            PriceSize::new(2.2,2.0),
        ]);

        PriceSizeBackLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert!(ps.capacity() > MIN_VEC_CAP);
        assert_eq!(ps, vec![
            PriceSize::new(1.1,6.0),
            PriceSize::new(1.2,5.0),
            PriceSize::new(1.3,4.0),
            PriceSize::new(1.4,3.0),
            PriceSize::new(1.5,2.0),
            PriceSize::new(1.6,1.0),
            PriceSize::new(1.7,2.0),
            PriceSize::new(1.8,2.0),
            PriceSize::new(1.9,2.0),
            PriceSize::new(2.0,2.0),
            PriceSize::new(2.1,2.0),
            PriceSize::new(2.2,2.0),
            PriceSize::new(2.3,3.0),
            PriceSize::new(2.4,3.0),
            PriceSize::new(2.5,3.0),
            PriceSize::new(2.6,3.0),
            PriceSize::new(2.7,3.0),
            PriceSize::new(2.8,3.0),
            PriceSize::new(2.9,3.0),
            PriceSize::new(3.0,3.0),
            PriceSize::new(3.1,3.0),
        ]);
    }
    
    #[test]
    fn test_lay_ladder_deserialize() {
        let raw = r#"
            [[5, 5],[3.0, 5],[2, 5],[6, 5],[4, 5]]
            [[1.5, 4],[3.5, 4],[2.5, 4.0],[1.25, 4],[7, 4]]
            [[7, 0],[1.5, 0.0],[5, 0],[1.25, 0]]
            [[2, 1],[2.5, 0],[3, 1],[3.5, 0],[4, 1],[5.0,1.0]]
            [[2, 0],[3, 0],[4.0, 0],[5, 0],[6.0, 0]]
            [[1.6, 1],[1.1, 6],[1.4, 3],[1.3, 4],[1.5, 2],[1.2, 5]]
            [[2.2, 2],[1.9, 2],[1.7, 2],[2.1, 2],[1.8, 2],[2.0, 2]]
            [[3.0, 3],[2.3, 3],[2.9, 3],[2.4, 3],[2.8, 3],[2.5, 3],[2.7, 3],[2.6, 3],[3.1, 3]]
        "#;
        let mut deser = serde_json::Deserializer::from_str(raw);
        
        let mut ps: Vec<PriceSize> = Vec::new();

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps.capacity(), MIN_VEC_CAP);        
        assert_eq!(ps, vec![
            PriceSize::new(6.0,5.0),
            PriceSize::new(5.0,5.0),
            PriceSize::new(4.0,5.0),
            PriceSize::new(3.0,5.0),
            PriceSize::new(2.0,5.0),
        ]);

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(7.0,4.0),
            PriceSize::new(6.0,5.0),
            PriceSize::new(5.0,5.0),
            PriceSize::new(4.0,5.0),
            PriceSize::new(3.5,4.0),
            PriceSize::new(3.0,5.0),
            PriceSize::new(2.5,4.0),
            PriceSize::new(2.0,5.0),
            PriceSize::new(1.5,4.0),
            PriceSize::new(1.25,4.0),
        ]);

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(6.0,5.0),
            PriceSize::new(4.0,5.0),
            PriceSize::new(3.5,4.0),
            PriceSize::new(3.0,5.0),
            PriceSize::new(2.5,4.0),
            PriceSize::new(2.0,5.0),
        ]);

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(6.0,5.0),
            PriceSize::new(5.0,1.0),
            PriceSize::new(4.0,1.0),
            PriceSize::new(3.0,1.0),
            PriceSize::new(2.0,1.0),
        ]);

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert!(ps.is_empty());
        
        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert_eq!(ps, vec![
            PriceSize::new(1.6,1.0),
            PriceSize::new(1.5,2.0),
            PriceSize::new(1.4,3.0),
            PriceSize::new(1.3,4.0),
            PriceSize::new(1.2,5.0),
            PriceSize::new(1.1,6.0),
        ]);

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert!(ps.capacity() == MIN_VEC_CAP);
        assert_eq!(ps, vec![
            PriceSize::new(2.2,2.0),
            PriceSize::new(2.1,2.0),
            PriceSize::new(2.0,2.0),
            PriceSize::new(1.9,2.0),
            PriceSize::new(1.8,2.0),
            PriceSize::new(1.7,2.0),
            PriceSize::new(1.6,1.0),
            PriceSize::new(1.5,2.0),
            PriceSize::new(1.4,3.0),
            PriceSize::new(1.3,4.0),
            PriceSize::new(1.2,5.0),
            PriceSize::new(1.1,6.0),
        ]);

        PriceSizeLayLadder(&mut ps).deserialize(&mut deser).expect("failed to deserialize");
        assert!(ps.capacity() > MIN_VEC_CAP);
        assert_eq!(ps, vec![
            PriceSize::new(3.1,3.0),
            PriceSize::new(3.0,3.0),
            PriceSize::new(2.9,3.0),
            PriceSize::new(2.8,3.0),
            PriceSize::new(2.7,3.0),
            PriceSize::new(2.6,3.0),
            PriceSize::new(2.5,3.0),
            PriceSize::new(2.4,3.0),
            PriceSize::new(2.3,3.0),
            PriceSize::new(2.2,2.0),
            PriceSize::new(2.1,2.0),
            PriceSize::new(2.0,2.0),
            PriceSize::new(1.9,2.0),
            PriceSize::new(1.8,2.0),
            PriceSize::new(1.7,2.0),
            PriceSize::new(1.6,1.0),
            PriceSize::new(1.5,2.0),
            PriceSize::new(1.4,3.0),
            PriceSize::new(1.3,4.0),
            PriceSize::new(1.2,5.0),
            PriceSize::new(1.1,6.0),
        ]);
    }

}

