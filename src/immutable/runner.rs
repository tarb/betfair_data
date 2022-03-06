// use pyo3::prelude::*;
// use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
// use serde::{Deserialize, Deserializer};
// use serde_json::value::RawValue;
// use std::{borrow::Cow, fmt};

// use crate::enums::SelectionStatus;
// use crate::ids::SelectionID;
// use crate::price_size::{F64OrStr, PriceSize};
// use super::datetime::DateTimeString;
// use super::price_size::{ImmutablePriceSizeBackLadder, ImmutablePriceSizeLayLadder};
// use super::config::Config;

// use crate::strings::StringSetExtNeq;

// #[pyclass(name = "Runner")]
// pub struct PyRunner {
//     #[pyo3(get)]
//     pub status: SelectionStatus,
//     #[pyo3(get)]
//     pub selection_id: SelectionID,
//     #[pyo3(get)]
//     pub name: String,
//     #[pyo3(get)]
//     pub last_price_traded: Option<f64>,
//     #[pyo3(get)]
//     pub total_matched: f64,
//     #[pyo3(get)]
//     pub adjustment_factor: Option<f64>,
//     #[pyo3(get)]
//     pub handicap: Option<f64>,
//     #[pyo3(get)]
//     pub ex: Py<PyRunnerBookEX>,
//     #[pyo3(get)]
//     pub sp: Py<PyRunnerBookSP>,
//     #[pyo3(get)]
//     pub sort_priority: u16,
//     #[pyo3(get)]
//     pub removal_date: Option<i64>,
//     // removal_date: Option<Py<PyDateTime>>,
//     pub removal_date_str: Option<DateTimeString>,
// }

// impl PyRunner {
//     fn new(py: Python) -> Self {
//         let ex: PyRunnerBookEX = Default::default();
//         let sp: PyRunnerBookSP = Default::default();

//         PyRunner {
//             selection_id: Default::default(),
//             status: Default::default(),
//             name: Default::default(),
//             last_price_traded: Default::default(),
//             total_matched: Default::default(),
//             adjustment_factor: Default::default(),
//             handicap: Default::default(),
//             sort_priority: Default::default(),
//             removal_date_str: Default::default(),
//             removal_date: Default::default(),
//             ex: Py::new(py, ex).unwrap(),
//             sp: Py::new(py, sp).unwrap(),
//         }
//     }

//     pub fn clone(&self, py: Python) -> Self {
//         let ex: PyRunnerBookEX = self.ex.borrow(py).clone();
//         let sp: PyRunnerBookSP = self.sp.borrow(py).clone();

//         Self {
//             selection_id: self.selection_id,
//             status: self.status,
//             name: self.name.clone(),
//             last_price_traded: self.last_price_traded,
//             total_matched: self.total_matched,
//             adjustment_factor: self.adjustment_factor,
//             handicap: self.handicap,
//             sort_priority: self.sort_priority,
//             removal_date_str: self.removal_date_str.clone(),
//             removal_date: self.removal_date,
//             ex: Py::new(py, ex).unwrap(),
//             sp: Py::new(py, sp).unwrap(),
//         }
//     }
// }

// pub struct PyRunnerDefSeq<'a, 'py> {
//     pub runners: &'a mut Vec<Py<PyRunner>>,
//     pub config: Config,
//     pub img: bool,
//     pub py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerDefSeq<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct RunnerSeqVisitor<'a, 'py> {
//             runners: &'a mut Vec<Py<PyRunner>>,
//             config: Config,
//             img: bool,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
//             type Value = ();

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//             where
//                 A: serde::de::SeqAccess<'de>,
//             {
//                 // grow an empty vec
//                 if self.runners.capacity() == 0 {
//                     match seq.size_hint() {
//                         Some(s) => self.runners.reserve_exact(s + 2),
//                         None => self.runners.reserve_exact(12),
//                     }
//                 }

//                 // WARNING
//                 // So this grossness, is due to us needing to get the selection_id to find the
//                 // runner to deserialize in place into. The id is not the first element of the
//                 // object, so we'd need to defer parsing the other properties, then come back
//                 // to them once we know the correct selection to use as the seed.
//                 // What we do here is parse the json twice, once pulling out only the id, then
//                 // again as normal
//                 #[derive(Deserialize)]
//                 struct RunnerWithID {
//                     id: SelectionID,
//                 }

//                 while let Some(raw) = seq.next_element::<&RawValue>()? {
//                     let mut deser = serde_json::Deserializer::from_str(raw.get());
//                     let rid: RunnerWithID =
//                         serde_json::from_str(raw.get()).map_err(Error::custom)?;

//                     let index = self
//                         .runners
//                         .iter()
//                         .map(|r| r.borrow_mut(self.py))
//                         .position(|r| r.selection_id == rid.id);
//                     match index {
//                         Some(index) => {
//                             let mut runner =
//                                 unsafe { self.runners.get_unchecked(index).borrow_mut(self.py) };
//                             PyRunnerDefinitonDeser {
//                                 runner: &mut runner,
//                                 config: self.config,
//                                 img: self.img,
//                                 py: self.py,
//                             }
//                             .deserialize(&mut deser)
//                             .map_err(Error::custom)?
//                         }
//                         None => {
//                             let mut runner = PyRunner::new(self.py);
//                             PyRunnerDefinitonDeser {
//                                 runner: &mut runner,
//                                 config: self.config,
//                                 img: self.img,
//                                 py: self.py,
//                             }
//                             .deserialize(&mut deser)
//                             .map_err(Error::custom)?;

//                             self.runners.push(Py::new(self.py, runner).unwrap());
//                         }
//                     }
//                 }

//                 // this config flag will reorder the runners into the order specified in the sort priority
//                 // as seen in the data files
//                 if !self.config.stable_runner_index {
//                     self.runners
//                         .sort_by_key(|r| r.borrow(self.py).sort_priority);
//                 }

//                 Ok(())
//             }
//         }

//         deserializer.deserialize_seq(RunnerSeqVisitor {
//             runners: self.runners,
//             config: self.config,
//             img: self.img,
//             py: self.py,
//         })
//     }
// }

// struct PyRunnerDefinitonDeser<'a, 'py> {
//     pub runner: &'a mut PyRunner,
//     pub config: Config,
//     pub img: bool,
//     pub py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerDefinitonDeser<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Debug, Deserialize)]
//         #[serde(field_identifier, rename_all = "camelCase")]
//         enum Field {
//             AdjustmentFactor,
//             Status,
//             SortPriority,
//             Id,
//             Name,
//             Bsp,
//             RemovalDate,
//             Hc,
//         }

//         struct RunnerDefVisitor<'a, 'py> {
//             runner: &'a mut PyRunner,
//             _config: Config,
//             _img: bool,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for RunnerDefVisitor<'a, 'py> {
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
//                         Field::Id => self.runner.selection_id = map.next_value()?,
//                         Field::AdjustmentFactor => {
//                             self.runner.adjustment_factor =
//                                 Some(map.next_value::<F64OrStr>()?.into())
//                         }
//                         Field::Status => self.runner.status = map.next_value()?,
//                         Field::SortPriority => self.runner.sort_priority = map.next_value()?,
//                         Field::Hc => {
//                             self.runner.handicap = Some(map.next_value::<F64OrStr>()?.into())
//                         }
//                         Field::Name => {
//                             self.runner.name.set_if_ne(map.next_value::<Cow<str>>()?);
//                         }
//                         Field::RemovalDate => {
//                             let s = map.next_value()?;
//                             if self.runner.removal_date_str.set_if_ne(s) {
//                                 let ts = chrono::DateTime::parse_from_rfc3339(s)
//                                     .map_err(Error::custom)?
//                                     .timestamp_millis();
//                                 self.runner.removal_date = Some(ts);
//                             }
//                         }
//                         Field::Bsp => {
//                             let mut sp = self.runner.sp.borrow_mut(self.py);
//                             sp.actual_sp = Some(map.next_value::<F64OrStr>()?.into());
//                         }
//                     }
//                 }

//                 Ok(())
//             }
//         }

//         const FIELDS: &[&str] = &[
//             "adjustmentFactor",
//             "status",
//             "sortPriority",
//             "id",
//             "name",
//             "bsp",
//             "removalDate",
//         ];
//         deserializer.deserialize_struct(
//             "RunnerDef",
//             FIELDS,
//             RunnerDefVisitor {
//                 runner: self.runner,
//                 _config: self.config,
//                 _img: self.img,
//                 py: self.py,
//             },
//         )
//     }
// }

// pub struct PyRunnerChangeSeq<'a, 'py> {
//     pub runners: &'a mut Vec<Py<PyRunner>>,
//     pub config: Config,
//     pub img: bool,
//     pub py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerChangeSeq<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct RunnerSeqVisitor<'a, 'py> {
//             runners: &'a mut Vec<Py<PyRunner>>,
//             config: Config,
//             img: bool,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
//             type Value = ();

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//             where
//                 A: serde::de::SeqAccess<'de>,
//             {
//                 // grow an empty vec
//                 if self.runners.capacity() == 0 {
//                     match seq.size_hint() {
//                         Some(s) => self.runners.reserve_exact(s + 2),
//                         None => self.runners.reserve_exact(12),
//                     }
//                 }

//                 #[derive(Deserialize)]
//                 struct RunnerWithID {
//                     id: SelectionID,
//                 }

//                 while let Some(raw) = seq.next_element::<&RawValue>()? {
//                     let mut deser = serde_json::Deserializer::from_str(raw.get());
//                     let rid: RunnerWithID =
//                         serde_json::from_str(raw.get()).map_err(Error::custom)?;

//                     let index = self
//                         .runners
//                         .iter()
//                         .map(|r| r.borrow_mut(self.py))
//                         .position(|r| r.selection_id == rid.id);
//                     match index {
//                         Some(index) => {
//                             let mut runner =
//                                 unsafe { self.runners.get_unchecked(index).borrow_mut(self.py) };
//                             PyRunnerChangeDeser {
//                                 runner: &mut runner,
//                                 img: self.img,                                
//                                 config: self.config,
//                                 py: self.py,
//                             }
//                             .deserialize(&mut deser)
//                             .map_err(Error::custom)?;
//                         }
//                         None => {
//                             let mut runner = PyRunner::new(self.py);
//                             PyRunnerChangeDeser {
//                                 runner: &mut runner,
//                                 config: self.config,
//                                 img: self.img,
//                                 py: self.py,
//                             }
//                             .deserialize(&mut deser)
//                             .map_err(Error::custom)?;

//                             self.runners.push(Py::new(self.py, runner).unwrap());
//                         }
//                     }
//                 }

//                 Ok(())
//             }
//         }

//         deserializer.deserialize_seq(RunnerSeqVisitor {
//             runners: self.runners,
//             py: self.py,
//             config: self.config,
//             img: self.img,
//         })
//     }
// }

// struct PyRunnerChangeDeser<'a, 'py> {
//     pub runner: &'a mut PyRunner,
//     pub config: Config,
//     pub img: bool,
//     pub py: Python<'py>,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerChangeDeser<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Debug, Deserialize)]
//         #[serde(field_identifier, rename_all = "camelCase")]
//         enum Field {
//             Id,
//             Atb,
//             Atl,
//             Spn,
//             Spf,
//             Spb,
//             Spl,
//             Trd,
//             Tv,
//             Ltp,
//             Hc,
//         }

//         struct RunnerChangeVisitor<'a, 'py> {
//             runner: &'a mut PyRunner,
//             config: Config,
//             img: bool,
//             py: Python<'py>,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for RunnerChangeVisitor<'a, 'py> {
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
//                         Field::Id => self.runner.selection_id = map.next_value()?,
//                         Field::Atb => {
//                             let mut ex = self.runner.ex.borrow_mut(self.py);
//                             let atb = &mut ex.available_to_back;
//                             if self.img {
//                                 atb.clear();
//                             }
//                             map.next_value_seed(PriceSizeLayLadder(atb))?;
//                         }
//                         Field::Atl => {
//                             let mut ex = self.runner.ex.borrow_mut(self.py);
//                             let atl = &mut ex.available_to_lay;
//                             if self.img {
//                                 atl.clear();
//                             }
//                             map.next_value_seed(PriceSizeBackLadder(atl))?;
//                         }
//                         Field::Trd => {
//                             let mut ex = self.runner.ex.borrow_mut(self.py);
//                             let trd = &mut ex.traded_volume;
//                             if self.img {
//                                 trd.clear();
//                             }

//                             map.next_value_seed(PriceSizeBackLadder(trd))?;

//                             if self.config.cumulative_runner_tv {
//                                 self.runner.total_matched =
//                                     ex.traded_volume.iter().map(|ps| ps.size).sum();
//                             }
//                         }
//                         Field::Spb => {
//                             let mut sp = self.runner.sp.borrow_mut(self.py);
//                             let spb = &mut sp.lay_liability_taken;
//                             if self.img {
//                                 spb.clear();
//                             }

//                             map.next_value_seed(PriceSizeLayLadder(spb))?;
//                         }
//                         Field::Spl => {
//                             let mut sp = self.runner.sp.borrow_mut(self.py);
//                             let spl = &mut sp.back_stake_taken;
//                             if self.img {
//                                 spl.clear();
//                             }

//                             map.next_value_seed(PriceSizeBackLadder(spl))?;
//                         }
//                         Field::Spn => {
//                             let mut sp = self.runner.sp.borrow_mut(self.py);
//                             sp.near_price = Some(map.next_value::<F64OrStr>()?.into());
//                         }
//                         Field::Spf => {
//                             let mut sp = self.runner.sp.borrow_mut(self.py);
//                             sp.far_price = Some(map.next_value::<F64OrStr>()?.into());
//                         }
//                         Field::Ltp => {
//                             self.runner.last_price_traded =
//                                 Some(map.next_value::<F64OrStr>()?.into())
//                         }
//                         Field::Hc => {
//                             self.runner.handicap = Some(map.next_value::<F64OrStr>()?.into())
//                         }
//                         // The betfair historic data files differ from the stream here, they send tv deltas
//                         // that need to be accumulated, whereas the stream sends the value itself.
//                         Field::Tv => {
//                             if self.config.cumulative_runner_tv {
//                                 map.next_value::<IgnoredAny>()?;
//                             } else {
//                                 self.runner.total_matched = map.next_value::<F64OrStr>()?.into();
//                             }
//                         }
//                     };
//                 }

//                 Ok(())
//             }
//         }

//         const FIELDS: &[&str] = &[
//             "id", "atb", "atl", "spn", "spf", "spb", "spl", "trd", "tv", "ltp", "hc",
//         ];
//         deserializer.deserialize_struct(
//             "RunnerChange",
//             FIELDS,
//             RunnerChangeVisitor {
//                 runner: self.runner,
//                 config: self.config,
//                 img: self.img,
//                 py: self.py,
//             },
//         )
//     }
// }
