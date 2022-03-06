// use log::warn;
// use pyo3::{exceptions, prelude::*};
// use serde::de::{Error, IgnoredAny};
// use serde::{
//     de::{self, DeserializeSeed, MapAccess, Visitor},
//     Deserialize, Deserializer,
// };
// use serde_json::value::RawValue;
// use std::borrow::Cow;
// use std::fmt;
// use std::path::PathBuf;
// use std::sync::Arc;

// use crate::deser::DeserializerWithData;
// use crate::enums::{MarketBettingType, MarketStatus};
// use crate::errors::DeserErr;
// use crate::ids::{EventID, EventTypeID, MarketID};
// use crate::market_source::SourceItem;
// use crate::mutable::runner::{PyRunner, PyRunnerChangeSeq, PyRunnerDefSeq};
// use crate::strings::{StringSetExtNeq, FixedSizeString};

// use super::config::Config;
// use super::container::SyncObj;
// use super::datetime::DateTimeString;

// #[pyclass(name = "MarketImage")]
// pub struct PyMarketBase {
//     #[pyo3(get)]
//     file: SyncObj<PathBuf>,
//     #[pyo3(get)]
//     bet_delay: u16,
//     #[pyo3(get)]
//     bsp_market: bool,
//     #[pyo3(get)]
//     bsp_reconciled: bool,
//     #[pyo3(get)]
//     clk: SyncObj<FixedSizeString<10>>,
//     #[pyo3(get)]
//     complete: bool,
//     #[pyo3(get)]
//     cross_matching: bool,
//     #[pyo3(get)]
//     discount_allowed: bool,
//     #[pyo3(get)]
//     each_way_divisor: Option<f64>,
//     #[pyo3(get)]
//     event_id: EventID,
//     #[pyo3(get)]
//     event_name: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     event_type_id: EventTypeID,
//     #[pyo3(get)]
//     in_play: bool,
//     #[pyo3(get)]
//     market_base_rate: u8,
//     #[pyo3(get)]
//     market_type: SyncObj<String>,
//     #[pyo3(get)]
//     market_name: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     number_of_active_runners: u16,
//     #[pyo3(get)]
//     number_of_winners: u8,
//     #[pyo3(get)]
//     persistence_enabled: bool,
//     #[pyo3(get)]
//     publish_time: u64,
//     #[pyo3(get)]
//     runners_voidable: bool,
//     #[pyo3(get)]
//     timezone: SyncObj<String>,
//     #[pyo3(get)]
//     total_matched: f64,
//     #[pyo3(get)]
//     turn_in_play_enabled: bool,
//     #[pyo3(get)]
//     venue: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     version: u64,
//     #[pyo3(get)]
//     runners: SyncObj<Arc<Vec<Py<PyRunner>>>>,
//     #[pyo3(get)]
//     status: MarketStatus,
//     #[pyo3(get)]
//     betting_type: MarketBettingType,
//     #[pyo3(get)]
//     market_time: SyncObj<DateTimeString>,
//     #[pyo3(get)]
//     open_date: SyncObj<DateTimeString>,
//     #[pyo3(get)]
//     suspend_time: Option<SyncObj<DateTimeString>>,
//     #[pyo3(get)]
//     settled_time: Option<SyncObj<DateTimeString>>,
//     #[pyo3(get)]
//     market_id: SyncObj<MarketID>,
//     #[pyo3(get)]
//     country_code: SyncObj<FixedSizeString<2>>,
// }

// impl PyMarketBase {
    
 




//     fn update(mut self_: PyRefMut<Self>, py: Python) -> PyResult<bool> {
//         let config = self_.config;
//         let mut deser = self_.deser.take().expect("Market without deser");
//         let base = self_.as_mut();

//         let r = Self::drive_deserialize(&mut deser, base, config, py)
//             .map(|_| true)
//             .unwrap_or_else(|err| {
//                 if !err.is_eof() {
//                     warn!(target: "betfair_data", "file: {} err: (JSON Parse Error) {}", base.file.to_string_lossy(), err);
//                 }

//                 false
//             });

//         self_.deser = Some(deser);

//         Ok(r)
//     }

//     fn copy(self_: PyRef<Self>, py: Python) -> PyResult<Py<PyMarketBase>> {
//         Py::new(py, self_.as_ref().clone(py))
//     }
// }

// struct PyMarketToken<'a, 'py> {
//     market: &'a mut PyMarketBase,
//     config: Config,
//     py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketToken<'a, 'py> {
//     type Value = ();

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

//         struct PyMarketOuterVisitor<'a, 'py> {
//             market: &'a mut PyMarketBase,
//             config: Config,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for PyMarketOuterVisitor<'a, 'py> {
//             type Value = ();

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Op => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                         Field::Pt => self.market.publish_time = map.next_value()?,
//                         Field::Mc => map.next_value_seed(PyMarketMcSeq {
//                             market: self.market,
//                             config: self.config,
//                             py: self.py,
//                         })?,
//                         Field::Clk => {
//                             self.market.clk.set_if_ne(map.next_value::<&str>()?);
//                         }
//                     }
//                 }

//                 Ok(())
//             }
//         }

//         const FIELDS: &[&str] = &["op", "pt", "clk", "mc"];
//         deserializer.deserialize_struct(
//             "MarketBook",
//             FIELDS,
//             PyMarketOuterVisitor {
//                 market: self.market,
//                 config: self.config,
//                 py: self.py,
//             },
//         )
//     }
// }

// // Used for serializing in place over the marketChange `mc` array
// struct PyMarketMcSeq<'a, 'py> {
//     market: &'a mut PyMarketBase,
//     config: Config,
//     py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketMcSeq<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct PyMarketMcSeqVisitor<'a, 'py> {
//             market: &'a mut PyMarketBase,
//             config: Config,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for PyMarketMcSeqVisitor<'a, 'py> {
//             type Value = ();

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//             where
//                 A: serde::de::SeqAccess<'de>,
//             {
//                 #[derive(Deserialize)]
//                 struct IDandImg {
//                     img: Option<bool>,
//                 }

//                 while let Some(raw) = seq.next_element::<&RawValue>()? {
//                     let mut deserializer = serde_json::Deserializer::from_str(raw.get());
//                     let idimg: IDandImg = serde_json::from_str(raw.get()).map_err(Error::custom)?;

//                     PyMarketMc {
//                         market: self.market,
//                         config: self.config,
//                         py: self.py,
//                         img: idimg.img.unwrap_or(false),
//                     }
//                     .deserialize(&mut deserializer)
//                     .map_err(Error::custom)?;
//                 }

//                 Ok(())
//             }
//         }

//         deserializer.deserialize_seq(PyMarketMcSeqVisitor {
//             market: self.market,
//             config: self.config,
//             py: self.py,
//         })
//     }
// }

// // Used for serializing in place over the marketChange `mc` objects
// struct PyMarketMc<'a, 'py> {
//     market: &'a mut PyMarketBase,
//     config: Config,
//     py: Python<'py>,
//     img: bool,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketMc<'a, 'py> {
//     type Value = ();

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

//         struct PyMarketMcVisitor<'a, 'py> {
//             market: &'a mut PyMarketBase,
//             config: Config,
//             img: bool,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for PyMarketMcVisitor<'a, 'py> {
//             type Value = ();

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Id => {
//                             // Do not currently support files that have multiple markets in them.
//                             // There are event level files that betfair provide, which have every
//                             // market for an event
//                             // with filtering: if a market has already been initted and then
//                             // changes then we must have a multi market file.
//                             let is_init = !self.market.market_id.is_empty();
//                             if self.market.market_id.set_if_ne(map.next_value::<&str>()?) && is_init
//                             {
//                                 return Err(Error::custom(
//                                     "multiple markets per file is not supported",
//                                 ));
//                             }
//                         }
//                         Field::MarketDefinition => map.next_value_seed(PyMarketDefinition {
//                             market: self.market,
//                             config: self.config,
//                             img: self.img,
//                             py: self.py,
//                         })?,
//                         Field::Rc => map.next_value_seed(PyRunnerChangeSeq {
//                             runners: &mut self.market.runners,
//                             img: self.img,                            
//                             config: self.config,
//                             py: self.py,
//                         })?,
//                         Field::Tv => {
//                             if !self.config.cumulative_runner_tv {
//                                 self.market.total_matched += map.next_value::<f64>()?;
//                             } else {
//                                 map.next_value::<IgnoredAny>()?;
//                             }
//                         }
//                         Field::Con => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                         Field::Img => {
//                             // TODO I need to handle this and clear the market
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                         _ => {
//                             map.next_value::<IgnoredAny>()?;
//                         }
//                     }
//                 }

//                 // if cumulative_runner_tv is on, then tv shouldnt be sent at a market level and will have
//                 // to be derived from the sum of runner tv's. This happens when using the data provided
//                 // from betfair historical data service, not saved from the actual stream
//                 if self.config.cumulative_runner_tv {
//                     self.market.total_matched = self
//                         .market
//                         .runners
//                         .iter()
//                         .map(|r| r.borrow(self.py).total_matched)
//                         .sum();
//                 }

//                 Ok(())
//             }
//         }

//         const FIELDS: &[&str] = &["id", "marketDefinition", "rc", "con", "img", "tv"];
//         deserializer.deserialize_struct(
//             "MarketChange",
//             FIELDS,
//             PyMarketMcVisitor {
//                 market: self.market,
//                 config: self.config,
//                 img: self.img,
//                 py: self.py,
//             },
//         )
//     }
// }

// // Used for serializing in place over the mc marketDefinition object
// struct PyMarketDefinition<'a, 'py> {
//     market: &'a mut PyMarketBase,
//     config: Config,
//     img: bool,
//     py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketDefinition<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Debug, Deserialize)]
//         #[serde(field_identifier, rename_all = "camelCase")]
//         enum Field {
//             BetDelay,
//             BettingType,
//             BspMarket,
//             BspReconciled,
//             Complete,
//             CountryCode,
//             CrossMatching,
//             DiscountAllowed,
//             EachWayDivisor,
//             EventId,
//             EventName,
//             EventTypeId,
//             InPlay,
//             KeyLineDefiniton,
//             LineMaxUnit,
//             LineMinUnit,
//             LineInterval,
//             MarketBaseRate,
//             MarketTime,
//             MarketType,
//             Name,
//             NumberOfActiveRunners,
//             NumberOfWinners,
//             OpenDate,
//             PersistenceEnabled,
//             PriceLadderDefinition,
//             RaceType,
//             Regulators,
//             Runners,
//             RunnersVoidable,
//             SettledTime,
//             Status,
//             SuspendTime,
//             Timezone,
//             TurnInPlayEnabled,
//             Venue,
//             Version,
//         }

//         struct PyMarketDefinitionVisitor<'a, 'py> {
//             market: &'a mut PyMarketBase,
//             config: Config,
//             img: bool,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for PyMarketDefinitionVisitor<'a, 'py> {
//             type Value = ();

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::BspMarket => self.market.bsp_market = map.next_value()?,
//                         Field::TurnInPlayEnabled => {
//                             self.market.turn_in_play_enabled = map.next_value()?
//                         }
//                         Field::InPlay => self.market.in_play = map.next_value()?,
//                         Field::PersistenceEnabled => {
//                             self.market.persistence_enabled = map.next_value()?
//                         }
//                         Field::BspReconciled => self.market.bsp_reconciled = map.next_value()?,
//                         Field::Complete => self.market.complete = map.next_value()?,
//                         Field::CrossMatching => self.market.cross_matching = map.next_value()?,
//                         Field::RunnersVoidable => {
//                             self.market.runners_voidable = map.next_value()?
//                         }
//                         Field::DiscountAllowed => {
//                             self.market.discount_allowed = map.next_value()?
//                         }
//                         Field::Timezone => {
//                             self.market.timezone.set_if_ne(map.next_value::<&str>()?);
//                         }
//                         Field::Name => {
//                             self.market
//                                 .market_name
//                                 .set_if_ne(map.next_value::<Cow<str>>()?);
//                         }
//                         Field::EventName => {
//                             self.market
//                                 .event_name
//                                 .set_if_ne(map.next_value::<Cow<str>>()?);
//                         }
//                         Field::CountryCode => {
//                             self.market
//                                 .country_code
//                                 .set_if_ne(map.next_value::<&str>()?);
//                         }
//                         Field::Venue => {
//                             self.market.venue.set_if_ne(map.next_value::<Cow<str>>()?);
//                         }
//                         Field::Status => self.market.status = map.next_value()?,
//                         Field::MarketBaseRate => {
//                             self.market.market_base_rate = map.next_value::<f32>()? as u8
//                         } // TODO: why is this needed
//                         Field::NumberOfWinners => {
//                             self.market.number_of_winners = map.next_value::<f32>()? as u8
//                         } // TODO: why is this needed
//                         Field::NumberOfActiveRunners => {
//                             self.market.number_of_active_runners = map.next_value()?
//                         }
//                         Field::BetDelay => self.market.bet_delay = map.next_value()?,
//                         Field::EventId => {
//                             self.market.event_id = map
//                                 .next_value::<&str>()?
//                                 .parse()
//                                 .map_err(de::Error::custom)?;
//                         }
//                         Field::EventTypeId => {
//                             self.market.event_type_id = map
//                                 .next_value::<&str>()?
//                                 .parse()
//                                 .map_err(de::Error::custom)?;
//                         }
//                         Field::Version => self.market.version = map.next_value()?,
//                         Field::Runners => map.next_value_seed(PyRunnerDefSeq {
//                             runners: &mut self.market.runners,
//                             config: self.config,
//                             img: self.img,
//                             py: self.py,
//                         })?,
//                         Field::MarketType => {
//                             self.market.market_type.set_if_ne(map.next_value::<&str>()?);
//                         }
//                         Field::BettingType => self.market.betting_type = map.next_value()?,
//                         Field::EachWayDivisor => {
//                             self.market.each_way_divisor = Some(map.next_value::<f64>()?)
//                         }
//                         Field::MarketTime => {
//                             let s = map.next_value()?;
//                             if self.market.market_time_str.set_if_ne(s) {
//                                 let ts = chrono::DateTime::parse_from_rfc3339(s)
//                                     .map_err(de::Error::custom)?
//                                     .timestamp_millis();

//                                 self.market.market_time = ts;
//                             }
//                         }
//                         Field::SuspendTime => {
//                             let s = map.next_value()?;
//                             if self.market.suspend_time_str.set_if_ne(s) {
//                                 let ts = chrono::DateTime::parse_from_rfc3339(s)
//                                     .map_err(de::Error::custom)?
//                                     .timestamp_millis();
//                                 // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
//                                 // self.market.suspend_time = Some(d.into_py(self.1));
//                                 self.market.suspend_time = Some(ts);
//                             }
//                         }
//                         Field::SettledTime => {
//                             let s = map.next_value()?;
//                             if self.market.settled_time_str.set_if_ne(s) {
//                                 let ts = chrono::DateTime::parse_from_rfc3339(s)
//                                     .map_err(de::Error::custom)?
//                                     .timestamp_millis();
//                                 // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
//                                 // self.market.settled_time = Some(d.into_py(self.1));
//                                 self.market.settled_time = Some(ts);
//                             }
//                         }
//                         Field::OpenDate => {
//                             let s = map.next_value()?;
//                             if self.market.open_date_str.set_if_ne(s) {
//                                 let ts = chrono::DateTime::parse_from_rfc3339(s)
//                                     .map_err(de::Error::custom)?
//                                     .timestamp_millis();
//                                 // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
//                                 // self.market.open_date = Some(d.into_py(self.1));
//                                 self.market.open_date = ts;
//                             }
//                         }
//                         Field::Regulators => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                         }

//                         // after searching over 200k markets, I cant find these values in any data sets :/
//                         Field::RaceType => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                             // panic!("{} {}", self.0.source, self.0.file);
//                         }
//                         Field::KeyLineDefiniton => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                             // panic!("{} {}", self.0.source, self.0.file);
//                         }
//                         Field::PriceLadderDefinition => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                             // panic!("{} {}", self.0.source, self.0.file);
//                         }
//                         Field::LineMaxUnit => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                             // panic!("{} {}", self.0.source, self.0.file);
//                         }
//                         Field::LineMinUnit => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                             // panic!("{} {}", self.0.source, self.0.file);
//                         }
//                         Field::LineInterval => {
//                             map.next_value::<serde::de::IgnoredAny>()?;
//                             // panic!("{} {}", self.0.source, self.0.file);
//                         }
//                     }
//                 }
//                 Ok(())
//             }
//         }

//         const FIELDS: &[&str] = &[
//             "keyLineDefiniton",
//             "priceLadderDefinition",
//             "raceType",
//             "lineMaxUnit",
//             "lineMinUnit",
//             "lineInterval",
//             "bspMarket",
//             "turnInPlayEnabled",
//             "persistenceEnabled",
//             "marketBaseRate",
//             "eventId",
//             "eventTypeId",
//             "numberOfWinners",
//             "bettingType",
//             "marketType",
//             "marketTime",
//             "suspendTime",
//             "bspReconciled",
//             "complete",
//             "inPlay",
//             "crossMatching",
//             "runnersVoidable",
//             "numberOfActiveRunners",
//             "betDelay",
//             "status",
//             "runners",
//             "regulators",
//             "countryCode",
//             "discountAllowed",
//             "timezone",
//             "openDate",
//             "version",
//             "name",
//             "eventName",
//             "venue",
//             "settledTime",
//             "eachWayDivisor",
//         ];
//         deserializer.deserialize_struct(
//             "MarketDefinition",
//             FIELDS,
//             PyMarketDefinitionVisitor {
//                 market: self.market,
//                 config: self.config,
//                 img: self.img,
//                 py: self.py,
//             },
//         )
//     }
// }

// #[cfg(test)]
// mod tests {

//     // test disabled awaiting merge which fixes cargo test
//     // https://github.com/PyO3/pyo3/pull/2135
//     /*
//     use super::*;

//     #[test]
//     fn test_multiple_markets() {
//         let mut m = PyMarketBase::new("".to_owned(), "".to_owned());
//         let py = unsafe { Python::assume_gil_acquired() };

//         let config = Config{cumulative_runner_tv: true, stable_runner_index: false};

//         let mut deser = serde_json::Deserializer::from_str(r#"{"id": "1.123456789"}{"id":"1.987654321"}"#);

//         PyMarketMc(&mut m, py, config).deserialize(&mut deser).expect("1st market_id deser ok");
//         PyMarketMc(&mut m, py, config).deserialize(&mut deser).expect_err("2nd market_id deser error");
//     }
//     */
// }
