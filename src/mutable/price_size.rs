use serde::de::{DeserializeSeed, Visitor};
use serde::Deserializer;
use std::cmp::Ordering;
use std::fmt;

use crate::price_size::PriceSize;

const MIN_VEC_CAP: usize = 20;

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
                    let cmp_fn = |ps2: &PriceSize| {
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
                            Ok(index) => {
                                self.0.remove(index);
                            }
                            Err(_err) => {}
                        }
                    } else {
                        match self.0.binary_search_by(cmp_fn) {
                            // updating price
                            Ok(index) => unsafe { self.0.get_unchecked_mut(index) }.size = ps1.size,
                            // inserting price
                            Err(index) => self.0.insert(index, ps1),
                        }
                    }
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(PSVisitor(self.0))
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
                    let cmp_fn = |ps2: &PriceSize| {
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
                            Ok(index) => {
                                self.0.remove(index);
                            }
                            Err(_err) => {}
                        }
                    } else {
                        match self.0.binary_search_by(cmp_fn) {
                            // updating price
                            Ok(index) => unsafe { self.0.get_unchecked_mut(index) }.size = ps1.size,
                            // inserting price
                            Err(index) => self.0.insert(index, ps1),
                        }
                    }
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(PSVisitor(self.0))
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

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(ps.capacity(), MIN_VEC_CAP);
        assert_eq!(
            ps,
            vec![
                PriceSize::new(2.0, 5.0),
                PriceSize::new(3.0, 5.0),
                PriceSize::new(4.0, 5.0),
                PriceSize::new(5.0, 5.0),
                PriceSize::new(6.0, 5.0),
            ]
        );

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(1.25, 4.0),
                PriceSize::new(1.5, 4.0),
                PriceSize::new(2.0, 5.0),
                PriceSize::new(2.5, 4.0),
                PriceSize::new(3.0, 5.0),
                PriceSize::new(3.5, 4.0),
                PriceSize::new(4.0, 5.0),
                PriceSize::new(5.0, 5.0),
                PriceSize::new(6.0, 5.0),
                PriceSize::new(7.0, 4.0),
            ]
        );

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(2.0, 5.0),
                PriceSize::new(2.5, 4.0),
                PriceSize::new(3.0, 5.0),
                PriceSize::new(3.5, 4.0),
                PriceSize::new(4.0, 5.0),
                PriceSize::new(6.0, 5.0),
            ]
        );

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(2.0, 1.0),
                PriceSize::new(3.0, 1.0),
                PriceSize::new(4.0, 1.0),
                PriceSize::new(5.0, 1.0),
                PriceSize::new(6.0, 5.0),
            ]
        );

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert!(ps.is_empty());

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(1.1, 6.0),
                PriceSize::new(1.2, 5.0),
                PriceSize::new(1.3, 4.0),
                PriceSize::new(1.4, 3.0),
                PriceSize::new(1.5, 2.0),
                PriceSize::new(1.6, 1.0),
            ]
        );

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert!(ps.capacity() == MIN_VEC_CAP);
        assert_eq!(
            ps,
            vec![
                PriceSize::new(1.1, 6.0),
                PriceSize::new(1.2, 5.0),
                PriceSize::new(1.3, 4.0),
                PriceSize::new(1.4, 3.0),
                PriceSize::new(1.5, 2.0),
                PriceSize::new(1.6, 1.0),
                PriceSize::new(1.7, 2.0),
                PriceSize::new(1.8, 2.0),
                PriceSize::new(1.9, 2.0),
                PriceSize::new(2.0, 2.0),
                PriceSize::new(2.1, 2.0),
                PriceSize::new(2.2, 2.0),
            ]
        );

        PriceSizeBackLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert!(ps.capacity() > MIN_VEC_CAP);
        assert_eq!(
            ps,
            vec![
                PriceSize::new(1.1, 6.0),
                PriceSize::new(1.2, 5.0),
                PriceSize::new(1.3, 4.0),
                PriceSize::new(1.4, 3.0),
                PriceSize::new(1.5, 2.0),
                PriceSize::new(1.6, 1.0),
                PriceSize::new(1.7, 2.0),
                PriceSize::new(1.8, 2.0),
                PriceSize::new(1.9, 2.0),
                PriceSize::new(2.0, 2.0),
                PriceSize::new(2.1, 2.0),
                PriceSize::new(2.2, 2.0),
                PriceSize::new(2.3, 3.0),
                PriceSize::new(2.4, 3.0),
                PriceSize::new(2.5, 3.0),
                PriceSize::new(2.6, 3.0),
                PriceSize::new(2.7, 3.0),
                PriceSize::new(2.8, 3.0),
                PriceSize::new(2.9, 3.0),
                PriceSize::new(3.0, 3.0),
                PriceSize::new(3.1, 3.0),
            ]
        );
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

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(ps.capacity(), MIN_VEC_CAP);
        assert_eq!(
            ps,
            vec![
                PriceSize::new(6.0, 5.0),
                PriceSize::new(5.0, 5.0),
                PriceSize::new(4.0, 5.0),
                PriceSize::new(3.0, 5.0),
                PriceSize::new(2.0, 5.0),
            ]
        );

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(7.0, 4.0),
                PriceSize::new(6.0, 5.0),
                PriceSize::new(5.0, 5.0),
                PriceSize::new(4.0, 5.0),
                PriceSize::new(3.5, 4.0),
                PriceSize::new(3.0, 5.0),
                PriceSize::new(2.5, 4.0),
                PriceSize::new(2.0, 5.0),
                PriceSize::new(1.5, 4.0),
                PriceSize::new(1.25, 4.0),
            ]
        );

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(6.0, 5.0),
                PriceSize::new(4.0, 5.0),
                PriceSize::new(3.5, 4.0),
                PriceSize::new(3.0, 5.0),
                PriceSize::new(2.5, 4.0),
                PriceSize::new(2.0, 5.0),
            ]
        );

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(6.0, 5.0),
                PriceSize::new(5.0, 1.0),
                PriceSize::new(4.0, 1.0),
                PriceSize::new(3.0, 1.0),
                PriceSize::new(2.0, 1.0),
            ]
        );

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert!(ps.is_empty());

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert_eq!(
            ps,
            vec![
                PriceSize::new(1.6, 1.0),
                PriceSize::new(1.5, 2.0),
                PriceSize::new(1.4, 3.0),
                PriceSize::new(1.3, 4.0),
                PriceSize::new(1.2, 5.0),
                PriceSize::new(1.1, 6.0),
            ]
        );

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert!(ps.capacity() == MIN_VEC_CAP);
        assert_eq!(
            ps,
            vec![
                PriceSize::new(2.2, 2.0),
                PriceSize::new(2.1, 2.0),
                PriceSize::new(2.0, 2.0),
                PriceSize::new(1.9, 2.0),
                PriceSize::new(1.8, 2.0),
                PriceSize::new(1.7, 2.0),
                PriceSize::new(1.6, 1.0),
                PriceSize::new(1.5, 2.0),
                PriceSize::new(1.4, 3.0),
                PriceSize::new(1.3, 4.0),
                PriceSize::new(1.2, 5.0),
                PriceSize::new(1.1, 6.0),
            ]
        );

        PriceSizeLayLadder(&mut ps)
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        assert!(ps.capacity() > MIN_VEC_CAP);
        assert_eq!(
            ps,
            vec![
                PriceSize::new(3.1, 3.0),
                PriceSize::new(3.0, 3.0),
                PriceSize::new(2.9, 3.0),
                PriceSize::new(2.8, 3.0),
                PriceSize::new(2.7, 3.0),
                PriceSize::new(2.6, 3.0),
                PriceSize::new(2.5, 3.0),
                PriceSize::new(2.4, 3.0),
                PriceSize::new(2.3, 3.0),
                PriceSize::new(2.2, 2.0),
                PriceSize::new(2.1, 2.0),
                PriceSize::new(2.0, 2.0),
                PriceSize::new(1.9, 2.0),
                PriceSize::new(1.8, 2.0),
                PriceSize::new(1.7, 2.0),
                PriceSize::new(1.6, 1.0),
                PriceSize::new(1.5, 2.0),
                PriceSize::new(1.4, 3.0),
                PriceSize::new(1.3, 4.0),
                PriceSize::new(1.2, 5.0),
                PriceSize::new(1.1, 6.0),
            ]
        );
    }
}
