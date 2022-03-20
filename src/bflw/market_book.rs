use pyo3::prelude::*;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::fmt;
use std::sync::Arc;

use super::market_definition::MarketDefinition;
use super::runner_book::RunnerBook;
use crate::bflw::market_definition::MarketDefinitionDeser;
use crate::bflw::runner_book::RunnerChangeSeq;
use crate::bflw::RoundToCents;
use crate::datetime::{DateTime, DateTimeString};
use crate::enums::MarketStatus;
use crate::ids::MarketID;
use crate::immutable::container::SyncObj;
use crate::market_source::SourceConfig;

#[pyclass]
pub struct MarketBook {
    #[pyo3(get)]
    pub publish_time: DateTime,
    #[pyo3(get)]
    pub bet_delay: u16,
    #[pyo3(get)]
    pub bsp_reconciled: bool,
    #[pyo3(get)]
    pub complete: bool,
    #[pyo3(get)]
    pub cross_matching: bool,
    #[pyo3(get)]
    pub inplay: bool,
    #[pyo3(get)]
    pub is_market_data_delayed: Option<bool>,
    #[pyo3(get)]
    pub number_of_active_runners: u16,
    #[pyo3(get)]
    pub number_of_runners: u16,
    #[pyo3(get)]
    pub number_of_winners: u8,
    #[pyo3(get)]
    pub runners_voidable: bool,
    #[pyo3(get)]
    pub status: MarketStatus,
    #[pyo3(get)]
    pub total_available: Option<()>, // f64 but bflw doesnt seem to use this on historic files
    #[pyo3(get)]
    pub total_matched: f64,
    #[pyo3(get)]
    pub version: u64,
    #[pyo3(get)]
    pub runners: SyncObj<Arc<Vec<Py<RunnerBook>>>>,
    #[pyo3(get)]
    pub market_definition: Py<MarketDefinition>,
    #[pyo3(get)]
    pub market_id: SyncObj<MarketID>,
    #[pyo3(get)]
    pub last_match_time: Option<SyncObj<DateTimeString>>,
}

#[derive(Default)]
struct MarketBookUpdate<'a> {
    market_id: &'a str,
    definition: Option<MarketDefinition>,
    runners: Option<Vec<Py<RunnerBook>>>,
    total_volume: Option<f64>,
}

#[pymethods]
impl MarketBook {
    #[getter(publish_time_epoch)]
    fn get_publish_time_epoch(&self, py: Python) -> PyObject {
        let ts = *self.publish_time;
        ts.into_py(py)
    }
}

impl MarketBook {
    fn new(change: MarketBookUpdate, py: Python) -> Self {
        let def = change.definition.unwrap(); // fix unwrap

        // maybe theres a better way to calculate this
        // let available = change
        //     .runners
        //     .as_ref()
        //     .map(|rs| {
        //         rs.iter()
        //             .map(|r| {
        //                 let r = r.borrow(py);
        //                 let ex = r.ex.borrow(py);
        //                 let back: f64 = ex.available_to_back.value.iter().map(|ps| ps.size).sum();
        //                 let lay: f64 = ex.available_to_lay.value.iter().map(|ps| ps.size).sum();
        //                 back + lay
        //             })
        //             .sum::<f64>()
        //     })
        //     .unwrap_or_default();

        Self {
            market_id: SyncObj::new(MarketID::try_from(change.market_id).unwrap()),
            runners: SyncObj::new(Arc::new(change.runners.unwrap_or_default())),
            total_matched: change.total_volume.unwrap_or_default(),
            bet_delay: def.bet_delay,
            bsp_reconciled: def.bsp_reconciled,
            complete: def.complete,
            cross_matching: def.cross_matching,
            inplay: def.in_play,
            is_market_data_delayed: None,
            number_of_active_runners: def.number_of_active_runners,
            number_of_runners: def.runners.len() as u16,
            runners_voidable: def.runners_voidable,
            status: def.status,
            number_of_winners: def.number_of_winners,
            version: def.version,
            total_available: None, // available,
            market_definition: Py::new(py, def).unwrap(),

            publish_time: DateTime::new(0),
            last_match_time: None,
        }
    }

    fn update_from_change(&self, change: MarketBookUpdate, py: Python) -> Self {
        // let available = change.runners.as_ref().map(|rs| {
        //     rs.iter()
        //         .map(|r| {
        //             let r = r.borrow(py);
        //             let ex = r.ex.borrow(py);
        //             let back: f64 = ex.available_to_back.value.iter().map(|ps| ps.size).sum();
        //             let lay: f64 = ex.available_to_lay.value.iter().map(|ps| ps.size).sum();
        //             back + lay
        //         })
        //         .sum::<f64>()
        // });

        Self {
            market_id: self.market_id.clone(),
            runners: change
                .runners
                .map(|r| SyncObj::new(Arc::new(r)))
                .unwrap_or_else(|| self.runners.clone()),
            total_matched: change.total_volume.unwrap_or(self.total_matched),
            bet_delay: change
                .definition
                .as_ref()
                .map(|def| def.bet_delay)
                .unwrap_or(self.bet_delay),
            bsp_reconciled: change
                .definition
                .as_ref()
                .map(|def| def.bsp_reconciled)
                .unwrap_or(self.bsp_reconciled),
            complete: change
                .definition
                .as_ref()
                .map(|def| def.complete)
                .unwrap_or(self.complete),
            cross_matching: change
                .definition
                .as_ref()
                .map(|def| def.cross_matching)
                .unwrap_or(self.cross_matching),
            inplay: change
                .definition
                .as_ref()
                .map(|def| def.in_play)
                .unwrap_or(self.inplay),
            is_market_data_delayed: None,
            number_of_active_runners: change
                .definition
                .as_ref()
                .map(|def| def.number_of_active_runners)
                .unwrap_or(self.number_of_active_runners),
            number_of_runners: change
                .definition
                .as_ref()
                .map(|def| def.runners.len() as u16)
                .unwrap_or(self.number_of_runners),
            runners_voidable: change
                .definition
                .as_ref()
                .map(|def| def.runners_voidable)
                .unwrap_or(self.runners_voidable),
            status: change
                .definition
                .as_ref()
                .map(|def| def.status)
                .unwrap_or(self.status),
            number_of_winners: change
                .definition
                .as_ref()
                .map(|def| def.number_of_winners)
                .unwrap_or(self.number_of_winners),
            version: change
                .definition
                .as_ref()
                .map(|def| def.version)
                .unwrap_or(self.version),
            total_available: None, // available.unwrap_or(self.total_available),
            market_definition: change
                .definition
                .map(|def| Py::new(py, def).unwrap())
                .unwrap_or_else(|| self.market_definition.clone()),

            publish_time: self.publish_time,
            last_match_time: None,
        }
    }
}

pub struct MarketBooksDeser<'a, 'py> {
    pub markets: &'a [Py<MarketBook>],
    pub py: Python<'py>,
    pub config: SourceConfig,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketBooksDeser<'a, 'py> {
    type Value = Vec<Py<MarketBook>>;

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

        struct MarketBooksDeserVisitor<'a, 'py> {
            markets: &'a [Py<MarketBook>],
            py: Python<'py>,
            config: SourceConfig,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketBooksDeserVisitor<'a, 'py> {
            type Value = Vec<Py<MarketBook>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pt: Option<DateTime> = None;
                let mut next_books: Vec<Py<MarketBook>> = Vec::new();

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Op => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Pt => {
                            pt = Some(DateTime::new(map.next_value::<u64>()?));
                        }
                        Field::Clk => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Mc => {
                            next_books = map.next_value_seed(MarketMcSeq {
                                markets: self.markets,
                                py: self.py,
                                config: self.config,
                            })?;
                        }
                    }
                }

                if let Some(pt) = pt {
                    next_books
                        .iter_mut()
                        .for_each(|mb| mb.borrow_mut(self.py).publish_time = pt);
                }

                // merge next_books and markets with missing clone_ref's
                for (i, mb) in self.markets.iter().enumerate() {
                    let rmb = mb.borrow(self.py);
                    let pos = next_books
                        .iter()
                        .position(|nmb| *nmb.borrow(self.py).market_id == *rmb.market_id);

                    match pos {
                        Some(ni) if i != ni => next_books.swap(i, ni),
                        None => {
                            next_books.push(mb.clone_ref(self.py));

                            let ni = next_books.len() - 1;
                            next_books.swap(i, ni);
                        }
                        // Some with ni and i in same location - do nothing
                        _ => {}
                    }
                }

                Ok(next_books)
            }
        }

        const FIELDS: &[&str] = &["op", "pt", "clk", "mc"];
        deserializer.deserialize_struct(
            "MarketBook",
            FIELDS,
            MarketBooksDeserVisitor {
                markets: self.markets,
                py: self.py,
                config: self.config,
            },
        )
    }
}

// Used for serializing in place over the marketChange `mc` array
struct MarketMcSeq<'a, 'py> {
    markets: &'a [Py<MarketBook>],
    py: Python<'py>,
    config: SourceConfig,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketMcSeq<'a, 'py> {
    type Value = Vec<Py<MarketBook>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MarketMcSeqVisitor<'a, 'py> {
            markets: &'a [Py<MarketBook>],
            py: Python<'py>,
            config: SourceConfig,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketMcSeqVisitor<'a, 'py> {
            type Value = Vec<Py<MarketBook>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                #[derive(Deserialize)]
                struct MarketWithID<'a> {
                    id: &'a str,
                    img: Option<bool>,
                }

                let mut next_books: Vec<Py<MarketBook>> = Vec::with_capacity(self.markets.len());

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let mid: MarketWithID =
                        serde_json::from_str(raw.get()).map_err(Error::custom)?;

                    let (mb, i) = {
                        let i = next_books
                            .iter()
                            .position(|m| (*m).borrow(self.py).market_id.as_str() == mid.id);

                        match i {
                            Some(i) if !mid.img.contains(&true) => {
                                (next_books.get(i).map(|m| m.borrow(self.py)), Some(i))
                            }
                            Some(i) if mid.img.contains(&true) => (None, Some(i)),
                            None if !mid.img.contains(&true) => (
                                self.markets
                                    .iter()
                                    .find(|m| (*m).borrow(self.py).market_id.as_str() == mid.id)
                                    .map(|o| o.borrow(self.py)),
                                None,
                            ),
                            _ => (None, None),
                        }
                    };

                    let next_mb = MarketMc {
                        market: mb,
                        py: self.py,
                        config: self.config,
                    }
                    .deserialize(&mut deser)
                    .map_err(Error::custom)?;

                    match i {
                        Some(i) => next_books[i] = Py::new(self.py, next_mb).unwrap(),
                        None => next_books.push(Py::new(self.py, next_mb).unwrap()),
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
    market: Option<PyRef<'py, MarketBook>>,
    py: Python<'py>,
    config: SourceConfig,
}
impl<'de, 'py> DeserializeSeed<'de> for MarketMc<'py> {
    type Value = MarketBook;

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
            market: Option<PyRef<'py, MarketBook>>,
            py: Python<'py>,
            config: SourceConfig,
        }
        impl<'de, 'py> Visitor<'de> for MarketMcVisitor<'py> {
            type Value = MarketBook;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut upt = MarketBookUpdate::default();
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            let s = map.next_value::<&str>()?;
                            upt.market_id = s;
                        }
                        Field::MarketDefinition => {
                            let def = self
                                .market
                                .as_ref()
                                .map(|mb| mb.market_definition.borrow(self.py));
                            let runners = upt
                                .runners
                                .as_ref()
                                .or_else(|| self.market.as_ref().map(|mb| mb.runners.as_ref()));

                            let (d, r) = map.next_value_seed(MarketDefinitionDeser {
                                def,
                                runners,
                                py: self.py,
                                config: self.config,
                            })?;

                            upt.definition = d;
                            upt.runners = r;
                        }
                        Field::Rc => {
                            let runners = upt
                                .runners
                                .as_ref()
                                .or_else(|| self.market.as_ref().map(|mb| mb.runners.as_ref()));
                            upt.runners = Some(map.next_value_seed(RunnerChangeSeq {
                                runners,
                                py: self.py,
                                config: self.config,
                            })?);

                            // if cumulative_runner_tv is on, then tv shouldnt be sent at a market level and will have
                            // to be derived from the sum of runner tv's. This happens when using the data provided
                            // from betfair historical data service, not saved from the actual stream
                            if self.config.cumulative_runner_tv {
                                upt.total_volume = upt
                                    .runners
                                    .as_ref()
                                    .map(|rs| {
                                        rs.iter().map(|r| r.borrow(self.py).total_matched).sum()
                                    })
                                    .map(|f: f64| f.round_cent());
                            }
                        }
                        Field::Tv => {
                            if !self.config.cumulative_runner_tv {
                                upt.total_volume = Some(map.next_value::<f64>()?.round_cent());
                            } else {
                                map.next_value::<IgnoredAny>()?;
                            }
                        }
                        Field::Con => {
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

                let mb = match (self.market, &upt.definition) {
                    (Some(mb), Some(_)) => mb.update_from_change(upt, self.py),
                    (Some(mb), None) => mb.update_from_change(upt, self.py),
                    (None, Some(_)) => MarketBook::new(upt, self.py),
                    (None, None) => {
                        return Err(Error::custom("missing definition on initial market update"))
                    }
                };

                Ok(mb)
            }
        }

        const FIELDS: &[&str] = &["id", "marketDefinition", "rc", "con", "img", "tv"];
        deserializer.deserialize_struct(
            "MarketChange",
            FIELDS,
            MarketMcVisitor {
                market: self.market,
                py: self.py,
                config: self.config,
            },
        )
    }
}
