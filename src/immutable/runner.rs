// use core::fmt;
// use pyo3::prelude::*;
// use pyo3::types::PyList;
// use serde::de::{DeserializeSeed, Error, Visitor};
// use serde::{Deserialize, Deserializer};
// use serde_json::value::RawValue;
// use std::borrow::Borrow;
// use std::sync::Arc;

// use super::container::{PyRep, SyncObj};
// use super::datetime::DateTimeString;
// use crate::enums::SelectionStatus;
// use crate::ids::SelectionID;
// use crate::market_source::SourceConfig;
// // use super::definition::RunnerDefUpdate;
// use super::runner_book_ex::RunnerBookEX;
// use super::runner_book_sp::RunnerBookSP;

// #[pyclass(name = "Runner")]
// pub struct PyRunner {
//     #[pyo3(get)]
//     pub status: SelectionStatus,
//     #[pyo3(get)]
//     pub selection_id: SelectionID,
//     #[pyo3(get)]
//     pub name: Option<SyncObj<Arc<String>>>,
//     #[pyo3(get)]
//     pub last_price_traded: Option<f64>,
//     #[pyo3(get)]
//     pub total_matched: f64,
//     #[pyo3(get)]
//     pub adjustment_factor: Option<f64>,
//     #[pyo3(get)]
//     pub handicap: Option<f64>,
//     #[pyo3(get)]
//     pub ex: Py<RunnerBookEX>,
//     #[pyo3(get)]
//     pub sp: Py<RunnerBookSP>,
//     #[pyo3(get)]
//     pub sort_priority: u16,
//     #[pyo3(get)]
//     pub removal_date: Option<SyncObj<DateTimeString>>,
// }

// // pub struct RunnerDefUpdate<'a> {
// //     id: SelectionID,
// //     adjustment_factor: Option<f64>,
// //     status: SelectionStatus,
// //     sort_priority: u16,
// //     name: Option<&'a str>,
// //     bsp: Option<F64OrStr>,
// //     removal_date: Option<&'a str>,
// //     hc: Option<F64OrStr>,
// // }

// impl PyRunner {}

// impl PyRep for Vec<Py<PyRunner>> {
//     fn py_rep(&self, py: Python) -> PyObject {
//         PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
//     }
// }

// pub struct RunnerChangeSeq<'a, 'py> {
//     pub runners: Option<&'a [Py<PyRunner>]>,
//     pub next: &'a mut Option<Vec<Py<PyRunner>>>,
//     pub py: Python<'py>,
//     pub config: SourceConfig,
// }

// impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeSeq<'a, 'py> {
//     type Value = ();

//     fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         struct RunnerSeqVisitor<'a, 'py> {
//             runners: Option<&'a Vec<Py<PyRunner>>>,
//             next: &'a mut Option<Vec<Py<PyRunner>>>,
//             py: Python<'py>,
//             config: SourceConfig,
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
//                 let mut v = self
//                     .runners
//                     .map(|v| v.iter().map(|r| r.clone_ref(self.py)).collect::<Vec<_>>())
//                     .unwrap_or_else(|| Vec::with_capacity(10));

//                 #[derive(Deserialize)]
//                 struct RunnerWithID {
//                     id: SelectionID,
//                 }

//                 while let Some(raw) = seq.next_element::<&RawValue>()? {
//                     let mut deser = serde_json::Deserializer::from_str(raw.get());
//                     let rid: RunnerWithID =
//                         serde_json::from_str(raw.get()).map_err(Error::custom)?;

//                     let runner = v
//                         .iter()
//                         .map(|r| r.borrow(self.py))
//                         .find(|r| r.selection_id == rid.id);

//                     let next_runner = self.next.and_then(|rs| {
//                         rs.iter_mut()
//                             .map(|r| r.borrow_mut(self.py))
//                             .find(|r| r.selection_id == rid.id)
//                     });

//                     match (runner, next_runner) {
//                         (Some(index), Some(next_runner)) => {
                         
//                                 RunnerBookChangeDeser {
//                                     runner: &runner,
//                                     next
//                                     py: self.py,
//                                     config: self.config,
//                                 }
//                                 .deserialize(&mut deser)
//                                 .map_err(Error::custom)?
//                             };

//                             v[index] = Py::new(self.py, runner).unwrap();
//                         }
//                         None => {
//                             let runner = RunnerBook::new(rid.id, self.py);
//                             let runner = RunnerBookChangeDeser {
//                                 runner: &runner,
//                                 py: self.py,
//                                 config: self.config,
//                             }
//                             .deserialize(&mut deser)
//                             .map_err(Error::custom)?;

//                             v.push(Py::new(self.py, runner).unwrap());
//                         }
//                     }
//                 }

//                 Ok(v)
//             }
//         }

//         deserializer.deserialize_seq(RunnerSeqVisitor {
//             runners: self.runners,
//             py: self.py,
//             config: self.config,
//         })
//     }
// }

// struct RunnerBookChangeDeser<'a, 'py> {
//     runner: &'a RunnerBook,
//     py: Python<'py>,
//     config: SourceConfig,
// }
// impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerBookChangeDeser<'a, 'py> {
//     type Value = RunnerBook;

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
//             runner: &'a RunnerBook,
//             py: Python<'py>,
//             config: SourceConfig,
//         }
//         impl<'de, 'a, 'py> Visitor<'de> for RunnerChangeVisitor<'a, 'py> {
//             type Value = RunnerBook;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("")
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut atb: Option<Vec<PriceSize>> = None;
//                 let mut atl: Option<Vec<PriceSize>> = None;
//                 let mut trd: Option<Vec<PriceSize>> = None;

//                 let mut spb: Option<Vec<PriceSize>> = None;
//                 let mut spl: Option<Vec<PriceSize>> = None;
//                 let mut spn: Option<FloatStr> = None;
//                 let mut spf: Option<FloatStr> = None;

//                 let mut tv: Option<f64> = None;
//                 let mut ltp: Option<FloatStr> = None;
//                 let mut hc: Option<FloatStr> = None;

//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Id => {
//                             let id = map.next_value::<SelectionID>()?;
//                             debug_assert!(id == self.runner.selection_id);
//                         }
//                         Field::Atb => {
//                             let ex = self.runner.ex.borrow(self.py);
//                             atb = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(
//                                 &ex.available_to_back,
//                             ))?);
//                         }
//                         Field::Atl => {
//                             let ex = self.runner.ex.borrow(self.py);
//                             atl = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(
//                                 &ex.available_to_lay,
//                             ))?);
//                         }
//                         Field::Trd => {
//                             let ex = self.runner.ex.borrow(self.py);
//                             let l = map
//                                 .next_value_seed(ImmutablePriceSizeBackLadder(&ex.traded_volume))?;

//                             if self.config.cumulative_runner_tv {
//                                 tv = Some(l.iter().map(|ps| ps.size).sum::<f64>().round_cent());
//                             }

//                             trd = Some(l);
//                         }
//                         Field::Spb => {
//                             let sp = self.runner.sp.borrow(self.py);
//                             spl = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(
//                                 &sp.lay_liability_taken,
//                             ))?);
//                         }
//                         Field::Spl => {
//                             let sp = self.runner.sp.borrow(self.py);
//                             spb = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(
//                                 &sp.back_stake_taken,
//                             ))?);
//                         }
//                         Field::Spn => {
//                             spn = Some(map.next_value::<FloatStr>()?);
//                         }
//                         Field::Spf => {
//                             spf = Some(map.next_value::<FloatStr>()?);
//                         }
//                         Field::Ltp => {
//                             ltp = Some(map.next_value::<FloatStr>()?);
//                         }
//                         Field::Hc => {
//                             hc = Some(map.next_value::<FloatStr>()?);
//                         }
//                         // The betfair historic data files differ from the stream here, they send tv deltas
//                         // that need to be accumulated, whereas the stream sends the value itself.
//                         Field::Tv => {
//                             if self.config.cumulative_runner_tv {
//                                 map.next_value::<IgnoredAny>()?;
//                             } else {
//                                 let v: f64 = map.next_value::<F64OrStr>()?.into();
//                                 let v = v.round_cent();
//                                 tv = Some(v);
//                             }
//                         }
//                     };
//                 }

//                 let ex = if atb.is_some() || atl.is_some() || trd.is_some() {
//                     let upt = RunnerBookEXUpdate {
//                         available_to_back: atb,
//                         available_to_lay: atl,
//                         traded_volume: trd,
//                     };

//                     Some(self.runner.ex.borrow(self.py).update(upt, self.py))
//                 } else {
//                     None
//                 };

//                 let sp = if spl.is_some() || spb.is_some() || spn.is_some() || spf.is_some() {
//                     let upt = RunnerBookSPUpdate {
//                         actual_sp: None,
//                         far_price: spf,
//                         near_price: spn,
//                         back_stake_taken: spb,
//                         lay_liability_taken: spl,
//                     };

//                     Some(self.runner.sp.borrow(self.py).update(upt, self.py))
//                 } else {
//                     None
//                 };

//                 let update = RunnerChangeUpdate {
//                     handicap: hc,
//                     last_price_traded: ltp,
//                     total_matched: tv,
//                     ex,
//                     sp,
//                 };

//                 Ok(self.runner.update_from_change(update, self.py))
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
//                 py: self.py,
//                 config: self.config,
//             },
//         )
//     }
// }
