#![allow(deprecated)]

use pyo3::{types::PyUnicode, Python, IntoPy, PyObject};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, IntoStaticStr};
use std::lazy::SyncOnceCell;


#[derive(
    Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize, AsRefStr, IntoStaticStr,
)]
pub enum MarketStatus {
    #[default]
    #[strum(serialize = "INACTIVE")]
    #[serde(rename = "INACTIVE")]
    InActive,
    #[strum(serialize = "OPEN")]
    #[serde(rename = "OPEN")]
    Open,
    #[strum(serialize = "SUSPENDED")]
    #[serde(rename = "SUSPENDED")]
    Suspended,
    #[strum(serialize = "CLOSED")]
    #[serde(rename = "CLOSED")]
    Closed,
}

static MARKET_STATUS_INTERNED: SyncOnceCell<[PyObject; std::mem::variant_count::<MarketStatus>()]> = SyncOnceCell::new();
impl IntoPy<PyObject> for MarketStatus {
    fn into_py(self, py: Python<'_>) -> PyObject {
        MARKET_STATUS_INTERNED.get_or_init(|| {
            [
                PyUnicode::new(py, MarketStatus::InActive.as_ref()).into_py(py),
                PyUnicode::new(py, MarketStatus::Open.as_ref()).into_py(py),
                PyUnicode::new(py, MarketStatus::Suspended.as_ref()).into_py(py),
                PyUnicode::new(py, MarketStatus::Closed.as_ref()).into_py(py),
            ]
        })
        [self as usize].clone_ref(py)
    }
}

#[derive(
    Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize, AsRefStr, IntoStaticStr,
)]
pub enum SelectionStatus {
    #[strum(serialize = "ACTIVE")]
    #[serde(rename = "ACTIVE")]
    Active,
    #[strum(serialize = "REMOVED")]
    #[serde(rename = "REMOVED")]
    Removed,
    #[strum(serialize = "REMOVED_VACANT")]
    #[serde(rename = "REMOVED_VACANT")]
    RemovedVacant,
    #[strum(serialize = "WINNER")]
    #[serde(rename = "WINNER")]
    Winner,
    #[strum(serialize = "PLACED")]
    #[serde(rename = "PLACED")]
    Placed,
    #[strum(serialize = "LOSER")]
    #[serde(rename = "LOSER")]
    Loser,
    #[default]
    #[strum(serialize = "HIDDEN")]
    #[serde(rename = "HIDDEN")]
    Hidden,
}

static SELECTION_STATUS_INTERNED: SyncOnceCell<[PyObject; std::mem::variant_count::<SelectionStatus>()]> = SyncOnceCell::new();
impl IntoPy<PyObject> for SelectionStatus {
    fn into_py(self, py: Python<'_>) -> PyObject {
        SELECTION_STATUS_INTERNED.get_or_init(|| {
            [
                PyUnicode::new(py, SelectionStatus::Active.as_ref()).into_py(py),
                PyUnicode::new(py, SelectionStatus::Removed.as_ref()).into_py(py),
                PyUnicode::new(py, SelectionStatus::RemovedVacant.as_ref()).into_py(py),
                PyUnicode::new(py, SelectionStatus::Winner.as_ref()).into_py(py),
                PyUnicode::new(py, SelectionStatus::Placed.as_ref()).into_py(py),
                PyUnicode::new(py, SelectionStatus::Loser.as_ref()).into_py(py),
                PyUnicode::new(py, SelectionStatus::Hidden.as_ref()).into_py(py),
            ]
        })
        [self as usize].clone_ref(py)
    }
}

#[derive(
    Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize, AsRefStr, IntoStaticStr,
)]
pub enum MarketBettingType {
    /// Odds Market - Any market that doesn't fit any any of the below categories.
    #[default]
    #[strum(serialize = "ODDS")]
    #[serde(rename = "ODDS")]
    Odds,
    /// Line Market - LINE markets operate at even-money odds of 2.0. However, price for these markets refers to the line positions available as defined by the markets min-max range and interval steps. Customers either Buy a line (LAY bet, winning if outcome is greater than the taken line (price)) or Sell a line (BACK bet, winning if outcome is less than the taken line (price)). If settled outcome equals the taken line, stake is returned.
    #[strum(serialize = "LINE")]
    #[serde(rename = "LINE")]
    Line,
    /// Range Market
    #[deprecated]
    #[strum(serialize = "RANGE")]
    #[serde(rename = "RANGE")]
    Range,
    /// Asian Handicap Market - A traditional Asian handicap market. Can be identified by marketType ASIAN_HANDICAP
    #[strum(serialize = "ASIAN_HANDICAP_DOUBLE_LINE")]
    #[serde(rename = "ASIAN_HANDICAP_DOUBLE_LINE")]
    AsianHandicapDoubleLine,
    /// Asian Single Line Market - A market in which there can be 0 or multiple winners. e,.g marketType TOTAL_GOALS
    #[strum(serialize = "ASIAN_HANDICAP_SINGLE_LINE")]
    #[serde(rename = "ASIAN_HANDICAP_SINGLE_LINE")]
    AsianHandicapSingleLine,
    /// Sportsbook Odds Market. This type is deprecated and will be removed in future releases, when Sportsbook markets will be represented as ODDS market but with a different product type.
    #[strum(serialize = "FIXED_ODDS")]
    #[serde(rename = "FIXED_ODDS")]
    FixedOdds,
}

static MARKET_BETTING_TYPE_INTERNED: SyncOnceCell<[PyObject; std::mem::variant_count::<MarketBettingType>()]> = SyncOnceCell::new();
impl IntoPy<PyObject> for MarketBettingType {
    fn into_py(self, py: Python<'_>) -> PyObject {
        MARKET_BETTING_TYPE_INTERNED.get_or_init(|| {
            [
                PyUnicode::new(py, MarketBettingType::Odds.as_ref()).into_py(py),
                PyUnicode::new(py, MarketBettingType::Line.as_ref()).into_py(py),
                PyUnicode::new(py, MarketBettingType::Range.as_ref()).into_py(py),
                PyUnicode::new(py, MarketBettingType::AsianHandicapDoubleLine.as_ref()).into_py(py),
                PyUnicode::new(py, MarketBettingType::AsianHandicapSingleLine.as_ref()).into_py(py),
                PyUnicode::new(py, MarketBettingType::FixedOdds.as_ref()).into_py(py),
            ]
        })
        [self as usize].clone_ref(py)
    }
}

#[derive(
    Debug, Default, PartialEq, Copy, Clone, Serialize, Deserialize, AsRefStr, IntoStaticStr,
)]
pub enum PriceLadderDefinition {
    #[default]
    #[strum(serialize = "CLASSIC")]
    #[serde(rename = "CLASSIC")]
    Classic,
    #[strum(serialize = "FINEST")]
    #[serde(rename = "FINEST")]
    Finest,
    #[deprecated]
    #[strum(serialize = "LINE_RANGE")]
    #[serde(rename = "LINE_RANGE")]
    LineRange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strum_static_str() {
        let s1: &'static str = "SUSPENDED";
        let s2: &'static str = MarketStatus::Suspended.as_ref();
        let s3: &'static str = MarketStatus::Suspended.into();

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
        assert_eq!(s1, s3);
    }
}
