use core::fmt;
use pyo3::prelude::*;
use pyo3::types::PyList;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::sync::Arc;

use super::container::SyncObj;
use super::runner_book_ex::RunnerBookEX;
use super::runner_book_sp::RunnerBookSP;
use crate::config::Config;
use crate::datetime::DateTimeString;
use crate::enums::SelectionStatus;
use crate::ids::SelectionID;
use crate::immutable::price_size::{ImmutablePriceSizeBackLadder, ImmutablePriceSizeLayLadder};
use crate::price_size::{F64OrStr, PriceSize};
use crate::py_rep::PyRep;

#[pyclass(name = "Runner")]
pub struct Runner {
    pub selection_id: SelectionID,
    #[pyo3(get)]
    pub status: SelectionStatus,
    #[pyo3(get)]
    pub name: Option<SyncObj<Arc<str>>>,
    #[pyo3(get)]
    pub last_price_traded: Option<f64>,
    #[pyo3(get)]
    pub total_matched: f64,
    #[pyo3(get)]
    pub adjustment_factor: Option<f64>,
    #[pyo3(get)]
    pub ex: Py<RunnerBookEX>,
    #[pyo3(get)]
    pub sp: Py<RunnerBookSP>,
    #[pyo3(get)]
    pub sort_priority: u16,
    #[pyo3(get)]
    pub removal_date: Option<SyncObj<DateTimeString>>,
}

#[pymethods]
impl Runner {
    #[getter(selection_id)]
    fn get_selection_id(&self) -> u32 {
        self.selection_id.id()
    }
    #[getter(handicap)]
    fn get_handicap(&self) -> Option<f32> {
        self.selection_id.handicap()
    }
}


impl PyRep for Vec<Py<Runner>> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

pub struct RunnerChangeSeq<'a, 'py> {
    pub runners: Option<&'a [Py<Runner>]>,
    pub next: Option<Vec<Py<Runner>>>,
    pub py: Python<'py>,
    pub config: Config,
}

impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeSeq<'a, 'py> {
    type Value = Option<Vec<Py<Runner>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            runners: Option<&'a [Py<Runner>]>,
            next: Option<Vec<Py<Runner>>>,
            py: Python<'py>,
            config: Config,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = Option<Vec<Py<Runner>>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                #[derive(Deserialize)]
                struct RunnerWithID {
                    id: u32,
                    hc: Option<f32>,
                }

                let mut next_runners = match self.next {
                    Some(n) => n,
                    None => self
                        .runners
                        .map(|v| v.iter().map(|r| r.clone_ref(self.py)).collect::<Vec<_>>())
                        .unwrap_or_else(|| Vec::with_capacity(10)),
                };

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let parts: RunnerWithID =
                        serde_json::from_str(raw.get()).map_err(Error::custom)?;
                    let sid = SelectionID::from((parts.id, parts.hc));

                    let index = next_runners
                        .iter()
                        .map(|r| r.borrow(self.py))
                        .position(|r| r.selection_id == sid);

                    match index {
                        Some(index) => {
                            let runner = RunnerChangeDeser {
                                id: sid,
                                runner: Some(unsafe {
                                    next_runners.get_unchecked(index).borrow(self.py)
                                }),
                                py: self.py,
                                config: self.config,
                            }
                            .deserialize(&mut deser)
                            .map_err(Error::custom)?;

                            next_runners[index] = Py::new(self.py, runner).unwrap();
                        }
                        None => {
                            let runner = RunnerChangeDeser {
                                id: sid,
                                runner: None,
                                py: self.py,
                                config: self.config,
                            }
                            .deserialize(&mut deser)
                            .map_err(Error::custom)?;

                            next_runners.push(Py::new(self.py, runner).unwrap());
                        }
                    }
                }

                Ok(Some(next_runners))
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor {
            runners: self.runners,
            next: self.next,
            py: self.py,
            config: self.config,
        })
    }
}

struct RunnerChangeDeser<'py> {
    id: SelectionID,
    runner: Option<PyRef<'py, Runner>>,
    py: Python<'py>,
    config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeDeser<'py> {
    type Value = Runner;

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

        struct RunnerChangeVisitor<'py> {
            id: SelectionID,
            runner: Option<PyRef<'py, Runner>>,
            py: Python<'py>,
            config: Config,
        }
        impl<'de, 'py> Visitor<'de> for RunnerChangeVisitor<'py> {
            type Value = Runner;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut upt = RunnerChangeUpdate::new(self.id);

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Atb => {
                            let ex = self.runner.as_ref().map(|r| r.ex.borrow(self.py));
                            let atb = ex.as_ref().map(|ex| ex.available_to_back.as_slice());

                            upt.atb = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(atb))?);
                        }
                        Field::Atl => {
                            let ex = self.runner.as_ref().map(|r| r.ex.borrow(self.py));
                            let atl = ex.as_ref().map(|ex| ex.available_to_lay.as_slice());

                            upt.atl = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(atl))?);
                        }
                        Field::Trd => {
                            let ex = self.runner.as_ref().map(|r| r.ex.borrow(self.py));
                            let trd = ex.as_ref().map(|ex| ex.traded_volume.as_slice());

                            let l = map.next_value_seed(ImmutablePriceSizeBackLadder(trd))?;

                            if self.config.cumulative_runner_tv {
                                upt.tv = Some(l.iter().map(|ps| ps.size).sum());
                            }

                            upt.trd = Some(l);
                        }
                        Field::Spb => {
                            let sp = self.runner.as_ref().map(|r| r.sp.borrow(self.py));
                            let spl = sp.as_ref().map(|sp| sp.lay_liability_taken.as_slice());

                            upt.spl = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(spl))?);
                        }
                        Field::Spl => {
                            let sp = self.runner.as_ref().map(|r| r.sp.borrow(self.py));
                            let spb = sp.as_ref().map(|sp| sp.back_stake_taken.as_slice());

                            upt.spb = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(spb))?);
                        }
                        Field::Spn => {
                            upt.spn = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Spf => {
                            upt.spf = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Ltp => {
                            upt.ltp = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Hc => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        // The betfair historic data files differ from the stream here, they send tv deltas
                        // that need to be accumulated, whereas the stream sends the value itself.
                        Field::Tv => {
                            if self.config.cumulative_runner_tv {
                                map.next_value::<IgnoredAny>()?;
                            } else {
                                let v: f64 = map.next_value::<F64OrStr>()?.into();
                                upt.tv = Some(v);
                            }
                        }
                    };
                }

                let pr = match self.runner {
                    Some(r) => upt.update(r, self.py),
                    None => upt.create(self.py),
                };

                Ok(pr)
            }
        }

        const FIELDS: &[&str] = &[
            "id", "atb", "atl", "spn", "spf", "spb", "spl", "trd", "tv", "ltp", "hc",
        ];
        deserializer.deserialize_struct(
            "RunnerChange",
            FIELDS,
            RunnerChangeVisitor {
                id: self.id,
                runner: self.runner,
                py: self.py,
                config: self.config,
            },
        )
    }
}

struct RunnerChangeUpdate {
    id: SelectionID,
    atb: Option<Vec<PriceSize>>,
    atl: Option<Vec<PriceSize>>,
    trd: Option<Vec<PriceSize>>,
    spb: Option<Vec<PriceSize>>,
    spl: Option<Vec<PriceSize>>,
    spn: Option<f64>,
    spf: Option<f64>,
    tv: Option<f64>,
    ltp: Option<f64>,
}

impl RunnerChangeUpdate {
    fn new(id: SelectionID) -> Self {
        Self {
            id, 
            atb: Default::default(),
            atl: Default::default(),
            trd: Default::default(),
            spb: Default::default(),
            spl: Default::default(),
            spn: Default::default(),
            spf: Default::default(),
            tv:  Default::default(),
            ltp: Default::default(),
        }
    }

    fn create(self, py: Python) -> Runner {
        let ex = Py::new(
            py,
            RunnerBookEX {
                available_to_back: SyncObj::new(Arc::new(self.atb.unwrap_or_default())),
                available_to_lay: SyncObj::new(Arc::new(self.atl.unwrap_or_default())),
                traded_volume: SyncObj::new(Arc::new(self.trd.unwrap_or_default())),
            },
        )
        .unwrap();

        let sp = Py::new(
            py,
            RunnerBookSP {
                actual_sp: None,
                far_price: self.spf,
                near_price: self.spn,
                back_stake_taken: SyncObj::new(Arc::new(self.spb.unwrap_or_default())),
                lay_liability_taken: SyncObj::new(Arc::new(self.spl.unwrap_or_default())),
            },
        )
        .unwrap();

        Runner {
            status: SelectionStatus::default(),
            selection_id: self.id,
            name: None,
            last_price_traded: self.ltp,
            total_matched: self.tv.unwrap_or_default(),
            adjustment_factor: None,
            ex,
            sp,
            sort_priority: 0,
            removal_date: None,
        }
    }

    fn update(self, runner: PyRef<Runner>, py: Python) -> Runner {
        let ex = if self.atb.is_some() || self.atl.is_some() || self.trd.is_some() {
            let ex = runner.ex.borrow(py);
            Py::new(
                py,
                RunnerBookEX {
                    available_to_back: self
                        .atb
                        .map(|atb| SyncObj::new(Arc::new(atb)))
                        .unwrap_or_else(|| ex.available_to_back.clone()),
                    available_to_lay: self
                        .atl
                        .map(|atl| SyncObj::new(Arc::new(atl)))
                        .unwrap_or_else(|| ex.available_to_lay.clone()),
                    traded_volume: self
                        .trd
                        .map(|trd| SyncObj::new(Arc::new(trd)))
                        .unwrap_or_else(|| ex.traded_volume.clone()),
                },
            )
            .unwrap()
        } else {
            runner.ex.clone_ref(py)
        };

        let sp =
            if self.spl.is_some() || self.spb.is_some() || self.spn.is_some() || self.spf.is_some()
            {
                let sp = runner.sp.borrow(py);
                Py::new(
                    py,
                    RunnerBookSP {
                        actual_sp: sp.actual_sp,
                        far_price: self.spf.or(sp.far_price),
                        near_price: self.spn.or(sp.near_price),
                        back_stake_taken: self
                            .spb
                            .map(|spb| SyncObj::new(Arc::new(spb)))
                            .unwrap_or_else(|| sp.back_stake_taken.clone()),
                        lay_liability_taken: self
                            .spl
                            .map(|spl| SyncObj::new(Arc::new(spl)))
                            .unwrap_or_else(|| sp.lay_liability_taken.clone()),
                    },
                )
                .unwrap()
            } else {
                runner.sp.clone_ref(py)
            };

        Runner {
            status: runner.status,
            selection_id: self.id,
            name: runner.name.clone(),
            last_price_traded: self.ltp.or(runner.last_price_traded),
            total_matched: self.tv.unwrap_or(runner.total_matched),
            adjustment_factor: runner.adjustment_factor,
            ex,
            sp,
            sort_priority: runner.sort_priority,
            removal_date: runner.removal_date.clone(),
        }
    }
}
