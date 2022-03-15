use serde::de::{DeserializeSeed, Visitor};
use serde::Deserializer;
use std::cmp::Ordering;
use std::fmt;

use crate::price_size::PriceSize;

pub struct ImmutablePriceSizeBackLadder<'a>(pub Option<&'a [PriceSize]>);
impl<'de, 'a> DeserializeSeed<'de> for ImmutablePriceSizeBackLadder<'a> {
    type Value = Vec<PriceSize>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct PSVisitor<'a>(Option<&'a [PriceSize]>);
        impl<'de, 'a> Visitor<'de> for PSVisitor<'a> {
            type Value = Vec<PriceSize>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("An array of PriceSize values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut v = match self.0 {
                    Some(ps) => {
                        let mut v = Vec::with_capacity(std::cmp::min(ps.len() + 5, 350));
                        ps.clone_into(&mut v);
                        v
                    }
                    None => Vec::with_capacity(10),
                };

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
                        match v.binary_search_by(cmp_fn) {
                            Ok(index) => {
                                v.remove(index);
                            }
                            Err(_err) => {}
                        }
                    } else {
                        match v.binary_search_by(cmp_fn) {
                            // updating price
                            Ok(index) => unsafe { v.get_unchecked_mut(index) }.size = ps1.size,
                            // inserting price
                            Err(index) => v.insert(index, ps1),
                        }
                    }
                }

                Ok(v)
            }
        }

        deserializer.deserialize_seq(PSVisitor(self.0))
    }
}

pub struct ImmutablePriceSizeLayLadder<'a>(pub Option<&'a [PriceSize]>);
impl<'de, 'a> DeserializeSeed<'de> for ImmutablePriceSizeLayLadder<'a> {
    type Value = Vec<PriceSize>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct PSVisitor<'a>(Option<&'a [PriceSize]>);
        impl<'de, 'a> Visitor<'de> for PSVisitor<'a> {
            type Value = Vec<PriceSize>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("An array of PriceSize values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut v = match self.0 {
                    Some(ps) => {
                        let mut v = Vec::with_capacity(std::cmp::min(ps.len() + 5, 350));
                        ps.clone_into(&mut v);
                        v
                    }
                    None => Vec::with_capacity(10),
                };

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
                        match v.binary_search_by(cmp_fn) {
                            Ok(index) => {
                                v.remove(index);
                            }
                            Err(_err) => {}
                        }
                    } else {
                        match v.binary_search_by(cmp_fn) {
                            // updating price
                            Ok(index) => unsafe { v.get_unchecked_mut(index) }.size = ps1.size,
                            // inserting price
                            Err(index) => v.insert(index, ps1),
                        }
                    }
                }

                Ok(v)
            }
        }

        deserializer.deserialize_seq(PSVisitor(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let ps1: Vec<PriceSize> = Vec::new();
        let ans1: Vec<PriceSize> = Vec::new();

        let ps2 = ImmutablePriceSizeBackLadder(Some(&ps1))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans2 = vec![
            PriceSize::new(2.0, 5.0),
            PriceSize::new(3.0, 5.0),
            PriceSize::new(4.0, 5.0),
            PriceSize::new(5.0, 5.0),
            PriceSize::new(6.0, 5.0),
        ];

        let ps3 = ImmutablePriceSizeBackLadder(Some(&ps2))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans3 = vec![
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
        ];

        let ps4 = ImmutablePriceSizeBackLadder(Some(&ps3))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans4 = vec![
            PriceSize::new(2.0, 5.0),
            PriceSize::new(2.5, 4.0),
            PriceSize::new(3.0, 5.0),
            PriceSize::new(3.5, 4.0),
            PriceSize::new(4.0, 5.0),
            PriceSize::new(6.0, 5.0),
        ];

        let ps5 = ImmutablePriceSizeBackLadder(Some(&ps4))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans5 = vec![
            PriceSize::new(2.0, 1.0),
            PriceSize::new(3.0, 1.0),
            PriceSize::new(4.0, 1.0),
            PriceSize::new(5.0, 1.0),
            PriceSize::new(6.0, 5.0),
        ];

        let ps6 = ImmutablePriceSizeBackLadder(Some(&ps5))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans6 = Vec::new();

        let ps7 = ImmutablePriceSizeBackLadder(Some(&ps6))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans7 = vec![
            PriceSize::new(1.1, 6.0),
            PriceSize::new(1.2, 5.0),
            PriceSize::new(1.3, 4.0),
            PriceSize::new(1.4, 3.0),
            PriceSize::new(1.5, 2.0),
            PriceSize::new(1.6, 1.0),
        ];

        let ps8 = ImmutablePriceSizeBackLadder(Some(&ps7))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans8 = vec![
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
        ];

        let ps9 = ImmutablePriceSizeBackLadder(Some(&ps8))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans9 = vec![
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
        ];

        // test at the end to ensure the immutability of the vec's
        assert_eq!(ps1, ans1);
        assert_eq!(ps2, ans2);
        assert_eq!(ps3, ans3);
        assert_eq!(ps4, ans4);
        assert_eq!(ps5, ans5);
        assert_eq!(ps6, ans6);
        assert_eq!(ps7, ans7);
        assert_eq!(ps8, ans8);
        assert_eq!(ps9, ans9);
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
            [[4.0, 5]]

        "#;
        let mut deser = serde_json::Deserializer::from_str(raw);

        let ps1: Vec<PriceSize> = Vec::new();
        let ans1: Vec<PriceSize> = Vec::new();

        let ps2 = ImmutablePriceSizeLayLadder(Some(&ps1))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans2 = vec![
            PriceSize::new(6.0, 5.0),
            PriceSize::new(5.0, 5.0),
            PriceSize::new(4.0, 5.0),
            PriceSize::new(3.0, 5.0),
            PriceSize::new(2.0, 5.0),
        ];

        let ps3 = ImmutablePriceSizeLayLadder(Some(&ps2))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans3 = vec![
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
        ];

        let ps4 = ImmutablePriceSizeLayLadder(Some(&ps3))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans4 = vec![
            PriceSize::new(6.0, 5.0),
            PriceSize::new(4.0, 5.0),
            PriceSize::new(3.5, 4.0),
            PriceSize::new(3.0, 5.0),
            PriceSize::new(2.5, 4.0),
            PriceSize::new(2.0, 5.0),
        ];

        let ps5 = ImmutablePriceSizeLayLadder(Some(&ps4))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans5 = vec![
            PriceSize::new(6.0, 5.0),
            PriceSize::new(5.0, 1.0),
            PriceSize::new(4.0, 1.0),
            PriceSize::new(3.0, 1.0),
            PriceSize::new(2.0, 1.0),
        ];

        let ps6 = ImmutablePriceSizeLayLadder(Some(&ps5))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans6 = Vec::new();

        let ps7 = ImmutablePriceSizeLayLadder(Some(&ps6))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans7 = vec![
            PriceSize::new(1.6, 1.0),
            PriceSize::new(1.5, 2.0),
            PriceSize::new(1.4, 3.0),
            PriceSize::new(1.3, 4.0),
            PriceSize::new(1.2, 5.0),
            PriceSize::new(1.1, 6.0),
        ];

        let ps8 = ImmutablePriceSizeLayLadder(Some(&ps7))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans8 = vec![
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
        ];

        let ps9 = ImmutablePriceSizeLayLadder(Some(&ps8))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans9 = vec![
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
        ];

        let ps10 = ImmutablePriceSizeLayLadder(Some(&ps9))
            .deserialize(&mut deser)
            .expect("failed to deserialize");
        let ans10 = vec![
            PriceSize::new(4.0, 5.0),
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
        ];

        assert_eq!(ps1, ans1);
        assert_eq!(ps2, ans2);
        assert_eq!(ps3, ans3);
        assert_eq!(ps4, ans4);
        assert_eq!(ps5, ans5);
        assert_eq!(ps6, ans6);
        assert_eq!(ps7, ans7);
        assert_eq!(ps8, ans8);
        assert_eq!(ps9, ans9);
        assert_eq!(ps10, ans10);
    }
}
