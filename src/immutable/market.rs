// use core::fmt;
// use std::borrow::Cow;
// use std::path::PathBuf;
// use std::sync::Arc;
// use pyo3::{prelude::*};
// use serde::{Deserializer, Deserialize};
// use serde::de::{DeserializeSeed, Visitor, MapAccess, IgnoredAny, Error};
// use serde_json::value::RawValue;

// use crate::enums::{MarketBettingType, MarketStatus};
// use crate::ids::{EventID, EventTypeID, MarketID};
// use crate::immutable::datetime::DateTime;
// use crate::immutable::definition::MarketDefinitionDeser;
// use crate::immutable::runner::RunnerChangeSeq;
// use crate::market_source::SourceConfig;
// use crate::strings::{FixedSizeString};
// use super::definition::MarketDefinitionUpdate;
// use super::runner::PyRunner;
// use super::container::SyncObj;
// use super::datetime::DateTimeString;

// #[pyclass(name = "MarketImage")]
// pub struct PyMarket {
//     #[pyo3(get)]
//     pub file: SyncObj<PathBuf>,
//     #[pyo3(get)]
//     pub bet_delay: u16,
//     #[pyo3(get)]
//     pub bsp_market: bool,
//     #[pyo3(get)]
//     pub bsp_reconciled: bool,
//     #[pyo3(get)]
//     pub clk: SyncObj<FixedSizeString<10>>,
//     #[pyo3(get)]
//     pub complete: bool,
//     #[pyo3(get)]
//     pub cross_matching: bool,
//     #[pyo3(get)]
//     pub discount_allowed: bool,
//     #[pyo3(get)]
//     pub each_way_divisor: Option<f64>,
//     #[pyo3(get)]
//     pub event_id: EventID,
//     #[pyo3(get)]
//     pub event_name: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     pub event_type_id: EventTypeID,
//     #[pyo3(get)]
//     pub in_play: bool,
//     #[pyo3(get)]
//     pub market_base_rate: f32,
//     #[pyo3(get)]
//     pub market_type: SyncObj<Arc<String>>,
//     #[pyo3(get)]
//     pub market_name: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     pub number_of_active_runners: u16,
//     #[pyo3(get)]
//     pub number_of_winners: u8,
//     #[pyo3(get)]
//     pub persistence_enabled: bool,
//     #[pyo3(get)]
//     pub publish_time: DateTime,
//     #[pyo3(get)]
//     pub runners_voidable: bool,
//     #[pyo3(get)]
//     pub timezone: SyncObj<Arc<String>>,
//     #[pyo3(get)]
//     pub total_matched: f64,
//     #[pyo3(get)]
//     pub turn_in_play_enabled: bool,
//     #[pyo3(get)]
//     pub venue: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     pub version: u64,
//     #[pyo3(get)]
//     pub runners: SyncObj<Arc<Vec<Py<PyRunner>>>>,
//     #[pyo3(get)]
//     pub status: MarketStatus,
//     #[pyo3(get)]
//     pub betting_type: MarketBettingType,
//     #[pyo3(get)]
//     pub market_time: SyncObj<DateTimeString>,
//     #[pyo3(get)]
//     pub open_date: SyncObj<DateTimeString>,
//     #[pyo3(get)]
//     pub suspend_time: Option<SyncObj<DateTimeString>>,
//     #[pyo3(get)]
//     pub settled_time: Option<SyncObj<DateTimeString>>,
//     #[pyo3(get)]
//     pub market_id: SyncObj<MarketID>,
//     #[pyo3(get)]
//     pub country_code: SyncObj<FixedSizeString<2>>,
//     #[pyo3(get)]
//     pub regulators: SyncObj<Arc<Vec<String>>>,

// }

// #[derive(Debug, Default)]
// struct MarketBookUpdate<'a> {
//     market_id: &'a str,
//     definition: Option<&'a MarketDefinitionUpdate<'a>>,
//     runners: Option<Vec<Py<PyRunner>>>,
//     total_volume: Option<f64>,
// }



// // we could detect multiple market ids and only do the 1 specified, and potentially repeat the deserializer
// // multiple times 1 for each market

// // or just intersperse the markets throughout the iter - probably better

// pub struct PyMarketsDeser<'a, 'py> {
//     pub markets: &'a [Py<PyMarket>],
//     pub py: Python<'py>,
//     pub config: SourceConfig,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketsDeser<'a, 'py> {
//     type Value = Vec<Py<PyMarket>>;

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Debug, Deserialize)]
//         #[serde(field_identifier, rename_all = "lowercase")]
//         enum Field {
//             Op,
//             Clk,
//             Pt,
//             Mc,
//         }

//         struct PyMarketsDeserVisitor<'a, 'py> {
//             markets: &'a [Py<PyMarket>],
//             py: Python<'py>,
//             config: SourceConfig,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for PyMarketsDeserVisitor<'a, 'py> {
//             type Value = Vec<Py<PyMarket>>;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut pt: Option<DateTime> = None;
//                 let mut clk: Option<FixedSizeString<10>> = None;
//                 let mut books: Vec<Py<PyMarket>> = Vec::new();

//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Pt => {
//                             pt = Some(DateTime::new(map.next_value::<u64>()?));
//                         }
//                         Field::Mc => {
//                             books = map.next_value_seed(MarketMcSeq {
//                                 markets: self.markets,
//                                 py: self.py,
//                                 config: self.config,
//                             })?;
//                         }
//                         Field::Clk => {
//                             clk = Some(map.next_value::<FixedSizeString<10>>()?);
//                         }
//                         Field::Op => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                     }
//                 }

//                 if let Some(pt) = pt {
//                     books
//                         .iter_mut()
//                         .for_each(|mb| mb.borrow_mut(self.py).publish_time = pt);
//                 }

//                 Ok(books)
//             }
//         }

//         const FIELDS: &[&str] = &["op", "pt", "clk", "mc"];
//         deserializer.deserialize_struct(
//             "PyMarket",
//             FIELDS,
//             PyMarketsDeserVisitor {
//                 markets: self.markets,
//                 py: self.py,
//                 config: self.config,
//             },
//         )
//     }
// }

// // Used for serializing in place over the marketChange `mc` array
// struct MarketMcSeq<'a, 'py> {
//     markets: &'a [Py<PyMarket>],
//     py: Python<'py>,
//     config: SourceConfig,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketMcSeq<'a, 'py> {
//     type Value = Vec<Py<PyMarket>>;

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct MarketMcSeqVisitor<'a, 'py> {
//             markets: &'a [Py<PyMarket>],
//             py: Python<'py>,
//             config: SourceConfig,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for MarketMcSeqVisitor<'a, 'py> {
//             type Value = Vec<Py<PyMarket>>;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//             where
//                 A: serde::de::SeqAccess<'de>,
//             {
//                 #[derive(Deserialize)]
//                 struct IdImg {
//                     id: MarketID,
//                     img: Option<bool>,
//                 }

//                 let mut next_books: Vec<Py<PyMarket>> = Vec::new();

//                 while let Some(raw) = seq.next_element::<&RawValue>()? {
//                     let mut deser = serde_json::Deserializer::from_str(raw.get());
//                     let mid: IdImg =
//                         serde_json::from_str(raw.get()).map_err(Error::custom)?;

//                     let mb = next_books
//                         .iter()
//                         // search already created markets
//                         .find(|m| (*m).borrow(self.py).market_id.as_str() == mid.id)
//                         // search markets passed in originally
//                         .or_else(|| {
//                             self.markets.iter().find(|m| {
//                                 (*m).borrow(self.py).market_id.as_str() == mid.id
//                             })
//                         })
//                         .map(|o| o.borrow(self.py));
                        
//                     let next_mb = MarketMc {
//                         id: mid.id,
//                         image: mid.img.contains(&true),
//                         market: mb,
//                         py: self.py,
//                         config: self.config,
//                     }
//                     .deserialize(&mut deser)
//                     .map_err(Error::custom)?;

//                     next_books.push(Py::new(self.py, next_mb).unwrap());
//                 }

//                 Ok(next_books)
//             }
//         }

//         deserializer.deserialize_seq(MarketMcSeqVisitor {
//             markets: self.markets,
//             py: self.py,
//             config: self.config,
//         })
//     }
// }

// struct MarketMc<'py> {
//     id: MarketID,
//     market: Option<PyRef<'py, PyMarket>>,
//     py: Python<'py>,
//     image: bool,
//     config: SourceConfig,
// }
// impl<'de, 'py> DeserializeSeed<'de> for MarketMc<'py> {
//     type Value = PyMarket;

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Debug, Deserialize)]
//         #[serde(field_identifier, rename_all = "camelCase")]
//         enum Field {
//             Id,
//             MarketDefinition,
//             Rc,
//             Con,
//             Img,
//             Tv,

//             // bflw recorded field
//             #[serde(rename = "_stream_id")]
//             StreamId,
//         }

//         struct MarketMcVisitor<'py> {
//             market: Option<PyRef<'py, PyMarket>>,
//             py: Python<'py>,
//             config: SourceConfig,
//         }
//         impl<'de, 'py> Visitor<'de> for MarketMcVisitor<'py> {
//             type Value = PyMarket;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut upt = MarketBookUpdate::default();

//                 let mut next: &mut Option<PyMarket> = &mut None;
//                 let mut next_runners: &mut Option<Vec<Py<PyRunner>>> = &mut None;

               
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Id => {
//                             let s = map.next_value::<&str>()?;
//                             upt.market_id = s;
//                         }
//                         Field::MarketDefinition => {
//                             map.next_value_seed(MarketDefinitionDeser {
//                                 market: self.market,
//                                 next,
//                                 next_runners,
//                                 py: self.py,
//                                 config: self.config,
//                             })?;
//                         }
//                         Field::Rc => {
//                             let runners: Option<&[Py<PyRunner>]> =
//                                 self.market.as_ref().map(|m| (**m.runners).as_ref());

//                             upt.runners = Some(map.next_value_seed(RunnerChangeSeq {
//                                 runners,
//                                 next: next_runners,
//                                 py: self.py,
//                                 config: self.config,
//                             })?);

//                             // if cumulative_runner_tv is on, then tv shouldnt be sent at a market level and will have
//                             // to be derived from the sum of runner tv's. This happens when using the data provided
//                             // from betfair historical data service, not saved from the actual stream
//                             if self.config.cumulative_runner_tv {
//                                 upt.total_volume = upt
//                                     .runners
//                                     .as_ref()
//                                     .map(|rs| {
//                                         rs.iter().map(|r| r.borrow(self.py).total_matched).sum()
//                                     })
//                                     .map(|f: f64| f.round_cent());
//                             }
//                         }
//                         Field::Tv => {
//                             if !self.config.cumulative_runner_tv {
//                                 upt.total_volume = Some(map.next_value::<f64>()?.round_cent());
//                             } else {
//                                 map.next_value::<IgnoredAny>()?;
//                             }
//                         }
//                         Field::Con => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                         Field::Img => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                         _ => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                     }
//                 }

//                 let mb = match (self.market, &upt.definition) {
//                     (Some(mb), Some(_)) => mb.update_from_change(upt, self.py),
//                     (Some(mb), None) => mb.update_from_change(upt, self.py),
//                     (None, Some(_)) => PyMarket::new(upt, self.py),
//                     (None, None) => {
//                         return Err(Error::custom("missing definition on initial market update"))
//                     }
//                 };

//                 Ok(mb)
//             }
//         }

//         const FIELDS: &[&str] = &["id", "marketDefinition", "rc", "con", "img", "tv"];
//         deserializer.deserialize_struct(
//             "MarketChange",
//             FIELDS,
//             MarketMcVisitor {
//                 market: self.market,
//                 py: self.py,
//                 config: self.config,
//             },
//         )
//     }
// }

