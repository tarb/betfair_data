use core::fmt;
use pyo3::prelude::*;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::collections::VecDeque;
use std::sync::Arc;

use super::container::SyncObj;
use super::definition::MarketDefinition;
use super::runner::Runner;
use crate::config::Config;
use crate::datetime::DateTime;
use crate::ids::{MarketID, Clk};
use crate::immutable::definition::MarketDefinitionDeser;
use crate::immutable::runner::RunnerChangeSeq;

#[derive(Clone)]
#[pyclass(name = "Market")]
pub struct Market {
    #[pyo3(get)]
    pub market_id: SyncObj<MarketID>,
    #[pyo3(get)]
    pub publish_time: DateTime,
    #[pyo3(get)]
    pub clk: SyncObj<Clk>,
    #[pyo3(get)]
    pub runners: SyncObj<Arc<Vec<Py<Runner>>>>,
    #[pyo3(get)]
    total_matched: f64,
    // uses getters to make the fields appear on the root market object
    def: Arc<MarketDefinition>,
}

#[pymethods]
impl Market {
    fn copy(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    #[getter(event_id)]
    fn get_event_id(&self, py: Python) -> PyObject {
        self.def.event_id.into_py(py)
    }
    #[getter(event_type_id)]
    fn get_event_type_id(&self, py: Python) -> PyObject {
        self.def.event_type_id.into_py(py)
    }
    #[getter(bet_delay)]
    fn get_bet_delay(&self, py: Python) -> PyObject {
        self.def.bet_delay.into_py(py)
    }
    #[getter(bsp_market)]
    fn get_bsp_market(&self, py: Python) -> PyObject {
        self.def.bsp_market.into_py(py)
    }
    #[getter(bsp_reconciled)]
    fn get_bsp_reconciled(&self, py: Python) -> PyObject {
        self.def.bsp_reconciled.into_py(py)
    }
    #[getter(complete)]
    fn get_complete(&self, py: Python) -> PyObject {
        self.def.complete.into_py(py)
    }
    #[getter(cross_matching)]
    fn get_cross_matching(&self, py: Python) -> PyObject {
        self.def.cross_matching.into_py(py)
    }
    #[getter(discount_allowed)]
    fn get_discount_allowed(&self, py: Python) -> PyObject {
        self.def.discount_allowed.into_py(py)
    }
    #[getter(each_way_divisor)]
    fn get_each_way_divisor(&self, py: Python) -> PyObject {
        self.def.each_way_divisor.into_py(py)
    }
    #[getter(event_name)]
    fn get_event_name(&self, py: Python) -> PyObject {
        self.def.event_name.to_object(py)
    }
    #[getter(in_play)]
    fn get_in_play(&self, py: Python) -> PyObject {
        self.def.in_play.into_py(py)
    }
    #[getter(market_base_rate)]
    fn get_market_base_rate(&self, py: Python) -> PyObject {
        self.def.market_base_rate.into_py(py)
    }
    #[getter(market_type)]
    fn get_market_type(&self, py: Python) -> PyObject {
        self.def.market_type.to_object(py)
    }
    #[getter(market_name)]
    fn get_market_name(&self, py: Python) -> PyObject {
        self.def.market_name.to_object(py)
    }
    #[getter(race_type)]
    fn get_race_type(&self, py: Python) -> PyObject {
        self.def.race_type.to_object(py)
    }
    #[getter(number_of_active_runners)]
    fn get_number_of_active_runners(&self, py: Python) -> PyObject {
        self.def.number_of_active_runners.into_py(py)
    }
    #[getter(number_of_winners)]
    fn get_number_of_winners(&self, py: Python) -> PyObject {
        self.def.number_of_winners.into_py(py)
    }
    #[getter(persistence_enabled)]
    fn get_persistence_enabled(&self, py: Python) -> PyObject {
        self.def.persistence_enabled.into_py(py)
    }
    #[getter(runners_voidable)]
    fn get_runners_voidable(&self, py: Python) -> PyObject {
        self.def.runners_voidable.into_py(py)
    }
    #[getter(timezone)]
    fn get_timezone(&self, py: Python) -> PyObject {
        self.def.timezone.to_object(py)
    }
    #[getter(turn_in_play_enabled)]
    fn get_turn_in_play_enabled(&self, py: Python) -> PyObject {
        self.def.turn_in_play_enabled.into_py(py)
    }
    #[getter(venue)]
    fn get_venue(&self, py: Python) -> PyObject {
        self.def.venue.to_object(py)
    }
    #[getter(version)]
    fn get_version(&self, py: Python) -> PyObject {
        self.def.version.into_py(py)
    }
    #[getter(status)]
    fn get_status(&self, py: Python) -> PyObject {
        self.def.status.into_py(py)
    }
    #[getter(betting_type)]
    fn get_betting_type(&self, py: Python) -> PyObject {
        self.def.betting_type.into_py(py)
    }
    #[getter(market_time)]
    fn get_market_time(&self, py: Python) -> PyObject {
        self.def.market_time.to_object(py)
    }
    #[getter(open_date)]
    fn get_open_date(&self, py: Python) -> PyObject {
        self.def.open_date.to_object(py)
    }
    #[getter(suspend_time)]
    fn get_suspend_time(&self, py: Python) -> PyObject {
        self.def.suspend_time.to_object(py)
    }
    #[getter(settled_time)]
    fn get_settled_time(&self, py: Python) -> PyObject {
        self.def.settled_time.to_object(py)
    }
    #[getter(country_code)]
    fn get_country_code(&self, py: Python) -> PyObject {
        self.def.country_code.to_object(py)
    }
    #[getter(regulators)]
    fn get_regulators(&self, py: Python) -> PyObject {
        self.def.regulators.to_object(py)
    }
}

pub struct MarketsDeser<'a, 'py> {
    pub markets: &'a [Py<Market>],
    pub py: Python<'py>,
    pub config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketsDeser<'a, 'py> {
    type Value = VecDeque<Py<Market>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Op,
            Clk,
            Pt,
            Mc,
        }

        struct MarketsDeserVisitor<'a, 'py> {
            markets: &'a [Py<Market>],
            py: Python<'py>,
            config: Config,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketsDeserVisitor<'a, 'py> {
            type Value = VecDeque<Py<Market>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pt: Option<DateTime> = None;
                let mut clk: Option<Clk> = None;
                let mut books: VecDeque<Py<Market>> = VecDeque::new();

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Pt => {
                            pt = Some(DateTime::new(map.next_value::<u64>()?));
                        }
                        Field::Mc => {
                            books = map.next_value_seed(MarketMcSeq {
                                markets: self.markets,
                                py: self.py,
                                config: self.config,
                            })?;
                        }
                        Field::Clk => {
                            clk = Some(map.next_value::<Clk>()?);
                        }
                        Field::Op => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                if let (Some(pt), Some(clk)) = (pt, clk) {
                    books.iter_mut().for_each(|mb| {
                        let mut m = mb.borrow_mut(self.py);
                        m.publish_time = pt;
                        m.clk = SyncObj::new(clk.clone());
                    });
                }

                Ok(books)
            }
        }

        const FIELDS: &[&str] = &["op", "pt", "clk", "mc"];
        deserializer.deserialize_struct(
            "Market",
            FIELDS,
            MarketsDeserVisitor {
                markets: self.markets,
                py: self.py,
                config: self.config,
            },
        )
    }
}

// Used for serializing in place over the marketChange `mc` array
struct MarketMcSeq<'a, 'py> {
    markets: &'a [Py<Market>],
    py: Python<'py>,
    config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketMcSeq<'a, 'py> {
    type Value = VecDeque<Py<Market>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MarketMcSeqVisitor<'a, 'py> {
            markets: &'a [Py<Market>],
            py: Python<'py>,
            config: Config,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketMcSeqVisitor<'a, 'py> {
            type Value = VecDeque<Py<Market>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                #[derive(Deserialize)]
                struct IdImg {
                    id: MarketID,
                    img: Option<bool>,
                }

                let mut next_books: VecDeque<Py<Market>> = VecDeque::new();

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let mid: IdImg = serde_json::from_str(raw.get()).map_err(Error::custom)?;

                    let (mb, i) = {
                        let i = next_books.iter()
                            .position(|m| (*m).borrow(self.py).market_id.as_str() == mid.id);

                        match i {
                            Some(i) if !mid.img.contains(&true) => {
                                (next_books.get(i).map(|m| m.borrow(self.py)), Some(i))
                            },
                            Some(i) if mid.img.contains(&true) => {
                                (None, Some(i))
                            },
                            None if !mid.img.contains(&true) => {
                                (self.markets.iter().find(|m| {
                                    (*m).borrow(self.py).market_id.as_str() == mid.id
                                })
                                .map(|o| o.borrow(self.py)), None)
                            }
                            _ => (None, None),
                        }  
                    };

                    let next_m = MarketMc {
                        id: mid.id,
                        market: mb,
                        py: self.py,
                        config: self.config,
                    }
                    .deserialize(&mut deser)
                    .map_err(Error::custom)?;

                    match (next_m, i) {
                        (Some(m), Some(i)) => next_books[i] = Py::new(self.py, m).unwrap(),
                        (Some(m), None) => next_books.push_back(Py::new(self.py, m).unwrap()),
                        _ => {}
                    }
                }

                Ok(next_books)
            }
        }

        deserializer.deserialize_seq(MarketMcSeqVisitor {
            markets: self.markets,
            py: self.py,
            config: self.config,
        })
    }
}

struct MarketMc<'py> {
    id: MarketID,
    market: Option<PyRef<'py, Market>>,
    py: Python<'py>,
    config: Config,
}
impl<'de, 'py> DeserializeSeed<'de> for MarketMc<'py> {
    type Value = Option<Market>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Id,
            MarketDefinition,
            Rc,
            Con,
            Img,
            Tv,

            // bflw recorded field
            #[serde(rename = "_stream_id")]
            StreamId,
        }

        struct MarketMcVisitor<'py> {
            id: MarketID,
            market: Option<PyRef<'py, Market>>,
            py: Python<'py>,
            config: Config,
        }
        impl<'de, 'py> Visitor<'de> for MarketMcVisitor<'py> {
            type Value = Option<Market>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut total_volume: Option<f64> = None;
                let mut next_def: Option<Arc<MarketDefinition>> = None;
                let mut next_runners: Option<Vec<Py<Runner>>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MarketDefinition => {
                            let def = self.market.as_ref().map(|m| &*m.def);
                            let runners = self
                                .market
                                .as_deref()
                                .map(|m| &m.runners)
                                .map(|rs| (**rs).as_slice());

                            (next_def, next_runners) =
                                map.next_value_seed(MarketDefinitionDeser {
                                    def,
                                    runners,
                                    next_runners,
                                    py: self.py,
                                    config: self.config,
                                })?;
                        }
                        Field::Rc => {
                            let runners: Option<&[Py<Runner>]> =
                                self.market.as_ref().map(|m| (**m.runners).as_ref());

                            next_runners = map.next_value_seed(RunnerChangeSeq {
                                runners,
                                next: next_runners,
                                py: self.py,
                                config: self.config,
                            })?;

                            // if cumulative_runner_tv is on, then tv shouldnt be sent at a market level and will have
                            // to be derived from the sum of runner tv's. This happens when using the data provided
                            // from betfair historical data service, not saved from the actual stream
                            if self.config.cumulative_runner_tv {
                                total_volume = next_runners.as_ref().map(|rs| {
                                    rs.iter().map(|r| r.borrow(self.py).total_matched).sum()
                                });
                            }
                        }
                        Field::Tv => {
                            if !self.config.cumulative_runner_tv {
                                total_volume = Some(map.next_value::<f64>()?);
                            } else {
                                map.next_value::<IgnoredAny>()?;
                            }
                        }
                        Field::Con => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Id => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Img => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let m = if let Some(market) = self.market {
                    Some(Market {
                        market_id: market.market_id.clone(),
                        publish_time: market.publish_time,
                        clk: market.clk.clone(),
                        total_matched: total_volume.unwrap_or(market.total_matched),
                        runners: next_runners
                            .map(|rs| SyncObj::new(Arc::new(rs)))
                            .unwrap_or_else(|| market.runners.clone()),
                        def: next_def.unwrap_or_else(|| market.def.clone()),
                    })
                } else {
                    Some(Market {
                        market_id: SyncObj::new(self.id),
                        publish_time: DateTime::new(0),
                        clk: Default::default(),
                        total_matched: total_volume.unwrap_or(0.0),
                        runners: next_runners
                            .map(|rs| SyncObj::new(Arc::new(rs)))
                            .ok_or_else(|| Error::custom("creating market without selections"))?,
                        def: next_def
                            .ok_or_else(|| Error::custom("creating market without definition"))?,
                    })
                };

                Ok(m)
            }
        }

        const FIELDS: &[&str] = &["id", "marketDefinition", "rc", "con", "img", "tv"];
        deserializer.deserialize_struct(
            "MarketChange",
            FIELDS,
            MarketMcVisitor {
                id: self.id,
                market: self.market,
                py: self.py,
                config: self.config,
            },
        )
    }
}
