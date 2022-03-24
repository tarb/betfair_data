use core::fmt;
use std::sync::Arc;
use pyo3::prelude::*;
use pyo3::types::PyList;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

use super::market_definition_runner::MarketDefRunnerUpdate;
use super::runner_book_sp::{RunnerBookSP};
use crate::bflw::float_str::FloatStr;
use crate::bflw::RoundToCents;
use crate::datetime::DateTimeString;
use crate::enums::SelectionStatus;
use crate::ids::SelectionID;
use crate::immutable::container::SyncObj;
use crate::immutable::price_size::{ImmutablePriceSizeBackLadder, ImmutablePriceSizeLayLadder};
use crate::immutable::runner_book_ex::{RunnerBookEX};
use crate::market_source::SourceConfig;
use crate::price_size::{F64OrStr, PriceSize};
use crate::py_rep::PyRep;

#[pyclass]
pub struct RunnerBook {
    pub selection_id: SelectionID,
    #[pyo3(get)]
    pub status: SelectionStatus,
    #[pyo3(get)]
    pub total_matched: f64,
    #[pyo3(get)]
    pub adjustment_factor: Option<f64>,
    #[pyo3(get)]
    pub last_price_traded: Option<FloatStr>,
    #[pyo3(get)]
    pub removal_date: Option<SyncObj<DateTimeString>>,
    #[pyo3(get)]
    pub ex: Py<RunnerBookEX>,
    #[pyo3(get)]
    pub sp: Py<RunnerBookSP>,
    #[pyo3(get)]
    pub matches: Vec<()>,
    #[pyo3(get)]
    pub orders: Vec<()>,
}

#[pymethods]
impl RunnerBook {
    #[getter(selection_id)]
    fn get_selection_id(&self) -> u32 {
        self.selection_id.id()
    }
    #[getter(handicap)]
    fn get_handicap(&self) -> FloatStr {
        let f = self.selection_id.handicap().unwrap_or(0.0);
        FloatStr(f as f64)
    }
}

pub struct RunnerChangeUpdate {
    last_price_traded: Option<FloatStr>,
    total_matched: Option<f64>,
    ex: Option<Py<RunnerBookEX>>,
    sp: Option<Py<RunnerBookSP>>,
}

impl PyRep for Vec<Py<RunnerBook>> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

impl RunnerBook {
    pub fn new(id: SelectionID, py: Python) -> Self {
        Self {
            selection_id: id,
            status: Default::default(),
            total_matched: Default::default(),
            adjustment_factor: Default::default(),
            last_price_traded: Default::default(),
            removal_date: Default::default(),
            ex: Py::new(py, RunnerBookEX::default()).unwrap(),
            sp: Py::new(py, RunnerBookSP::default()).unwrap(),
            matches: Default::default(),
            orders: Default::default(),
        }
    }

    pub fn update_from_change(&self, change: RunnerChangeUpdate, py: Python) -> Self {
        Self {
            selection_id: self.selection_id,
            adjustment_factor: self.adjustment_factor,
            status: self.status,
            removal_date: self.removal_date.clone(),
            last_price_traded: change.last_price_traded.or(self.last_price_traded),
            total_matched: change.total_matched.unwrap_or(self.total_matched),
            ex: change.ex.unwrap_or_else(|| self.ex.clone_ref(py)),
            sp: change.sp.unwrap_or_else(|| self.sp.clone_ref(py)),

            matches: self.matches.clone(), // always empty
            orders: self.orders.clone(),   // always empty
        }
    }

    pub fn would_change(&self, change: &MarketDefRunnerUpdate, py: Python) -> bool {
        self.status != change.status
            || self.adjustment_factor != change.adjustment_factor
            || self.sp.borrow(py).actual_sp != change.bsp
            || ((self.removal_date.is_none() && change.removal_date.is_some())
                || self
                    .removal_date
                    .is_some_and(|s| !change.removal_date.contains(&s.as_str())))
    }

    pub fn update_from_def(&self, change: &MarketDefRunnerUpdate, py: Python) -> Self {
        // need to update sp obj with bsp value if it's changed
        let sp = {
            let sp = self.sp.borrow(py);
            if sp.actual_sp != change.bsp {
                Py::new(py, RunnerBookSP {
                    actual_sp: change.bsp,
                    far_price: sp.far_price,
                    near_price: sp.near_price,
                    back_stake_taken: sp.back_stake_taken.clone(),
                    lay_liability_taken: sp.lay_liability_taken.clone(),
                }).unwrap()
            } else {
                self.sp.clone_ref(py)
            }
        };

        Self {
            selection_id: self.selection_id,
            status: change.status,
            adjustment_factor: change.adjustment_factor, //.or(self.adjustment_factor),
            last_price_traded: self.last_price_traded,
            total_matched: self.total_matched,
            ex: self.ex.clone_ref(py),
            sp,
            removal_date: change
                .removal_date
                .and_then(|s| match &self.removal_date {
                    Some(rd) if rd.as_str() != s => {
                        let dts = DateTimeString::new(s).unwrap(); // TODO: fix unwrap, maybe runner def update should take the dt already passed
                        Some(SyncObj::new(dts))
                    }
                    None => {
                        let dts = DateTimeString::new(s).unwrap();
                        Some(SyncObj::new(dts))
                    }
                    _ => self.removal_date.clone(),
                }),
                // .or_else(|| self.removal_date.clone()),

            matches: self.matches.clone(), // always empty
            orders: self.orders.clone(),   // always empty
        }
    }
}

pub struct RunnerChangeSeq<'a, 'py> {
    pub runners: Option<&'a Vec<Py<RunnerBook>>>,
    pub py: Python<'py>,
    pub config: SourceConfig,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeSeq<'a, 'py> {
    type Value = Vec<Py<RunnerBook>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            runners: Option<&'a Vec<Py<RunnerBook>>>,
            py: Python<'py>,
            config: SourceConfig,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = Vec<Py<RunnerBook>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut v = self
                    .runners
                    .map(|v| v.iter().map(|r| r.clone_ref(self.py)).collect::<Vec<_>>())
                    .unwrap_or_else(|| Vec::with_capacity(10));

                #[derive(Deserialize)]
                struct RunnerWithID {
                    id: u32,
                    hc: Option<f32>,
                }

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let parts: RunnerWithID =
                        serde_json::from_str(raw.get()).map_err(Error::custom)?;
                    let sid = SelectionID::from((parts.id, parts.hc));

                    let index = v
                        .iter()
                        .map(|r| r.borrow(self.py))
                        .position(|r| r.selection_id == sid);

                    match index {
                        Some(index) => {
                            let runner = {
                                let runner = unsafe { v.get_unchecked(index).borrow(self.py) };
                                RunnerBookChangeDeser {
                                    runner: &runner,
                                    py: self.py,
                                    config: self.config,
                                }
                                .deserialize(&mut deser)
                                .map_err(Error::custom)?
                            };

                            v[index] = Py::new(self.py, runner).unwrap();
                        }
                        None => {
                            let runner = RunnerBook::new(sid, self.py);
                            let runner = RunnerBookChangeDeser {
                                runner: &runner,
                                py: self.py,
                                config: self.config,
                            }
                            .deserialize(&mut deser)
                            .map_err(Error::custom)?;

                            v.push(Py::new(self.py, runner).unwrap());
                        }
                    }
                }

                Ok(v)
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor {
            runners: self.runners,
            py: self.py,
            config: self.config,
        })
    }
}

struct RunnerBookChangeDeser<'a, 'py> {
    runner: &'a RunnerBook,
    py: Python<'py>,
    config: SourceConfig,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerBookChangeDeser<'a, 'py> {
    type Value = RunnerBook;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Id,
            Atb,
            Atl,
            Spn,
            Spf,
            Spb,
            Spl,
            Trd,
            Tv,
            Ltp,
            Hc,
        }

        struct RunnerChangeVisitor<'a, 'py> {
            runner: &'a RunnerBook,
            py: Python<'py>,
            config: SourceConfig,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerChangeVisitor<'a, 'py> {
            type Value = RunnerBook;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut atb: Option<Vec<PriceSize>> = None;
                let mut atl: Option<Vec<PriceSize>> = None;
                let mut trd: Option<Vec<PriceSize>> = None;

                let mut spb: Option<Vec<PriceSize>> = None;
                let mut spl: Option<Vec<PriceSize>> = None;
                let mut spn: Option<FloatStr> = None;
                let mut spf: Option<FloatStr> = None;

                let mut tv: Option<f64> = None;
                let mut ltp: Option<FloatStr> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Atb => {
                            let ex = self.runner.ex.borrow(self.py);
                            atb = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(Some(
                                &ex.available_to_back,
                            )))?);
                        }
                        Field::Atl => {
                            let ex = self.runner.ex.borrow(self.py);
                            atl = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(Some(
                                &ex.available_to_lay,
                            )))?);
                        }
                        Field::Trd => {
                            let ex = self.runner.ex.borrow(self.py);
                            let l = map.next_value_seed(ImmutablePriceSizeBackLadder(Some(
                                &ex.traded_volume,
                            )))?;

                            if self.config.cumulative_runner_tv {
                                tv = Some(l.iter().map(|ps| ps.size).sum::<f64>().round_cent());
                            }

                            trd = Some(l);
                        }
                        Field::Spb => {
                            let sp = self.runner.sp.borrow(self.py);
                            spl = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(Some(
                                &sp.lay_liability_taken,
                            )))?);
                        }
                        Field::Spl => {
                            let sp = self.runner.sp.borrow(self.py);
                            spb = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(Some(
                                &sp.back_stake_taken,
                            )))?);
                        }
                        Field::Spn => {
                            spn = Some(map.next_value::<FloatStr>()?);
                        }
                        Field::Spf => {
                            spf = Some(map.next_value::<FloatStr>()?);
                        }
                        Field::Ltp => {
                            ltp = Some(map.next_value::<FloatStr>()?);
                        }
                        Field::Hc => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Id => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        // The betfair historic data files differ from the stream here, they send tv deltas
                        // that need to be accumulated, whereas the stream sends the value itself.
                        Field::Tv => {
                            if self.config.cumulative_runner_tv {
                                map.next_value::<IgnoredAny>()?;
                            } else {
                                let v: f64 = map.next_value::<F64OrStr>()?.into();
                                let v = v.round_cent();
                                tv = Some(v);
                            }
                        }
                    };
                }

                let ex = if atb.is_some() || atl.is_some() || trd.is_some() {
                    let ex = self.runner.ex.borrow(self.py);
                    Some(Py::new(self.py,RunnerBookEX {
                        available_to_back: atb.map_or_else(
                            || ex.available_to_back.clone(),
                            |ps| SyncObj::new(Arc::new(ps)),
                        ),
                        available_to_lay: atl.map_or_else(
                            || ex.available_to_lay.clone(),
                            |ps| SyncObj::new(Arc::new(ps)),
                        ),
                        traded_volume: trd.map_or_else(
                            || ex.traded_volume.clone(),
                            |ps| SyncObj::new(Arc::new(ps)),
                        ),
                    }).unwrap())

                } else {
                    None
                };

                let sp = if spl.is_some() || spb.is_some() || spn.is_some() || spf.is_some() {
                    let sp = self.runner.sp.borrow(self.py);
                    Some(Py::new(self.py, RunnerBookSP {
                        actual_sp: sp.actual_sp,
                        far_price: spf.or(sp.far_price),
                        near_price: spn.or(sp.near_price),
                        back_stake_taken: spb.map_or_else(
                            || sp.back_stake_taken.clone(),
                            |ps| SyncObj::new(Arc::new(ps)),
                        ),
                        lay_liability_taken: spl.map_or_else(
                            || sp.lay_liability_taken.clone(),
                            |ps| SyncObj::new(Arc::new(ps)),
                        ),
                    }).unwrap())
                } else {
                    None
                };

                let update = RunnerChangeUpdate {
                    last_price_traded: ltp,
                    total_matched: tv,
                    ex,
                    sp,
                };

                Ok(self.runner.update_from_change(update, self.py))
            }
        }

        const FIELDS: &[&str] = &[
            "id", "atb", "atl", "spn", "spf", "spb", "spl", "trd", "tv", "ltp", "hc",
        ];
        deserializer.deserialize_struct(
            "RunnerChange",
            FIELDS,
            RunnerChangeVisitor {
                runner: self.runner,
                py: self.py,
                config: self.config,
            },
        )
    }
}
