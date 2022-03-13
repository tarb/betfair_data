use core::fmt;
use pyo3::prelude::*;
use pyo3::types::PyList;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::sync::Arc;

use super::container::{PyRep, SyncObj};
use crate::datetime::DateTimeString;
use crate::enums::SelectionStatus;
use crate::ids::SelectionID;
use crate::immutable::price_size::{ImmutablePriceSizeBackLadder, ImmutablePriceSizeLayLadder};
use crate::config::Config;
use crate::price_size::{F64OrStr, PriceSize};
use super::runner_book_ex::RunnerBookEX;
use super::runner_book_sp::RunnerBookSP;

#[pyclass(name = "Runner")]
pub struct PyRunner {
    #[pyo3(get)]
    pub status: SelectionStatus,
    #[pyo3(get)]
    pub selection_id: SelectionID,
    #[pyo3(get)]
    pub name: Option<SyncObj<Arc<String>>>,
    #[pyo3(get)]
    pub last_price_traded: Option<f64>,
    #[pyo3(get)]
    pub total_matched: f64,
    #[pyo3(get)]
    pub adjustment_factor: Option<f64>,
    #[pyo3(get)]
    pub handicap: Option<f64>,
    #[pyo3(get)]
    pub ex: Py<RunnerBookEX>,
    #[pyo3(get)]
    pub sp: Py<RunnerBookSP>,
    #[pyo3(get)]
    pub sort_priority: u16,
    #[pyo3(get)]
    pub removal_date: Option<SyncObj<DateTimeString>>,
}

// pub struct RunnerDefUpdate<'a> {
//     id: SelectionID,
//     adjustment_factor: Option<f64>,
//     status: SelectionStatus,
//     sort_priority: u16,
//     name: Option<&'a str>,
//     bsp: Option<F64OrStr>,
//     removal_date: Option<&'a str>,
//     hc: Option<F64OrStr>,
// }

impl PyRunner {}

impl PyRep for Vec<Py<PyRunner>> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

pub struct RunnerChangeSeq<'a, 'py> {
    pub runners: Option<&'a [Py<PyRunner>]>,
    pub next: Option<Vec<Py<PyRunner>>>,
    pub py: Python<'py>,
    pub image: bool,
    pub config: Config,
}

impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeSeq<'a, 'py> {
    type Value = Option<Vec<Py<PyRunner>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            runners: Option<&'a [Py<PyRunner>]>,
            next: Option<Vec<Py<PyRunner>>>,
            py: Python<'py>,
            image: bool,
            config: Config,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = Option<Vec<Py<PyRunner>>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                #[derive(Deserialize)]
                struct RunnerWithID {
                    id: SelectionID,
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
                    let rid: RunnerWithID =
                        serde_json::from_str(raw.get()).map_err(Error::custom)?;

                    let index = next_runners
                        .iter()
                        .map(|r| r.borrow(self.py))
                        .position(|r| r.selection_id == rid.id);

                    match index {
                        Some(index) => {
                            let runner = RunnerChangeDeser {
                                runner: Some(unsafe {
                                    next_runners.get_unchecked(index).borrow(self.py)
                                }),
                                py: self.py,
                                image: self.image,
                                config: self.config,
                            }
                            .deserialize(&mut deser)
                            .map_err(Error::custom)?;

                            next_runners[index] = Py::new(self.py, runner).unwrap();
                        }
                        None => {
                            let runner = RunnerChangeDeser {
                                runner: None,
                                py: self.py,
                                image: self.image,
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
            image: self.image,
            config: self.config,
        })
    }
}

struct RunnerChangeDeser<'py> {
    runner: Option<PyRef<'py, PyRunner>>,
    py: Python<'py>,
    image: bool,
    config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeDeser<'py> {
    type Value = PyRunner;

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
            runner: Option<PyRef<'py, PyRunner>>,
            py: Python<'py>,
            image: bool,
            config: Config,
        }
        impl<'de, 'py> Visitor<'de> for RunnerChangeVisitor<'py> {
            type Value = PyRunner;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut upt = RunnerChangeUpdate::default();

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            upt.id = map.next_value::<SelectionID>()?;
                        }
                        Field::Atb => {
                            let ex = self.runner.as_ref().map(|r| r.ex.borrow(self.py));
                            let atb = ex
                                .as_ref()
                                .and_then(|ex| (!self.image).then(|| &ex.available_to_back))
                                .map(|atb| (**atb).as_slice());

                            upt.atb = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(atb))?);
                        }
                        Field::Atl => {
                            let ex = self.runner.as_ref().map(|r| r.ex.borrow(self.py));
                            let atl = ex
                                .as_ref()
                                .and_then(|ex| (!self.image).then(|| &ex.available_to_lay))
                                .map(|atl| (**atl).as_slice());

                            upt.atl = Some(map.next_value_seed(ImmutablePriceSizeBackLadder(atl))?);
                        }
                        Field::Trd => {
                            let ex = self.runner.as_ref().map(|r| r.ex.borrow(self.py));
                            let trd = ex
                                .as_ref()
                                .and_then(|ex| (!self.image).then(|| &ex.traded_volume))
                                .map(|trd| (**trd).as_slice());

                            let l = map.next_value_seed(ImmutablePriceSizeBackLadder(trd))?;
                            if self.config.cumulative_runner_tv {
                                upt.tv = Some(l.iter().map(|ps| ps.size).sum::<f64>());
                            }

                            upt.trd = Some(l);
                        }
                        Field::Spb => {
                            let sp = self.runner.as_ref().map(|r| r.sp.borrow(self.py));
                            let spl = sp
                                .as_ref()
                                .and_then(|sp| (!self.image).then(|| &sp.lay_liability_taken))
                                .map(|spl| (**spl).as_slice());
                            upt.spl = Some(map.next_value_seed(ImmutablePriceSizeLayLadder(spl))?);
                        }
                        Field::Spl => {
                            let sp = self.runner.as_ref().map(|r| r.sp.borrow(self.py));
                            let spb = sp
                                .as_ref()
                                .and_then(|sp| (!self.image).then(|| &sp.back_stake_taken))
                                .map(|spb| (**spb).as_slice());
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
                            upt.hc = Some(map.next_value::<F64OrStr>()?.into());
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
                    Some(r) => upt.update(r, self.image, self.py),
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
                runner: self.runner,
                py: self.py,
                image: self.image,
                config: self.config,
            },
        )
    }
}

#[derive(Default)]
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
    hc: Option<f64>,
}

impl RunnerChangeUpdate {
    fn create(self, py: Python) -> PyRunner {
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

        PyRunner {
            status: SelectionStatus::default(),
            selection_id: self.id,
            name: None,
            last_price_traded: None,
            total_matched: self.tv.unwrap_or_default(),
            adjustment_factor: None,
            handicap: self.hc,
            ex,
            sp,
            sort_priority: 0,
            removal_date: None,
        }
    }

    fn update(self, runner: PyRef<PyRunner>, image: bool, py: Python) -> PyRunner {
        let ex = if self.atb.is_some() || self.atl.is_some() || self.trd.is_some() {
            let ex = runner.ex.borrow(py);
            Py::new(
                py,
                RunnerBookEX {
                    available_to_back: self
                        .atb
                        .map(|atb| SyncObj::new(Arc::new(atb)))
                        .unwrap_or_else(|| {
                            image
                                .then(SyncObj::default)
                                .unwrap_or_else(|| ex.available_to_back.clone())
                        }),
                    available_to_lay: self
                        .atl
                        .map(|atl| SyncObj::new(Arc::new(atl)))
                        .unwrap_or_else(|| {
                            image
                                .then(SyncObj::default)
                                .unwrap_or_else(|| ex.available_to_lay.clone())
                        }),
                    traded_volume: self
                        .trd
                        .map(|trd| SyncObj::new(Arc::new(trd)))
                        .unwrap_or_else(|| {
                            image
                                .then(SyncObj::default)
                                .unwrap_or_else(|| ex.traded_volume.clone())
                        }),
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
                        far_price: self.spf.or({
                            if !image {
                                sp.far_price
                            } else {
                                None
                            }
                        }),
                        near_price: self.spn.or({
                            if !image {
                                sp.near_price
                            } else {
                                None
                            }
                        }),
                        back_stake_taken: self
                            .spb
                            .map(|spb| SyncObj::new(Arc::new(spb)))
                            .unwrap_or_else(|| {
                                image
                                    .then(SyncObj::default)
                                    .unwrap_or_else(|| sp.back_stake_taken.clone())
                            }),
                        lay_liability_taken: self
                            .spl
                            .map(|spl| SyncObj::new(Arc::new(spl)))
                            .unwrap_or_else(|| {
                                image
                                    .then(SyncObj::default)
                                    .unwrap_or_else(|| sp.lay_liability_taken.clone())
                            }),
                    },
                )
                .unwrap()
            } else {
                runner.sp.clone_ref(py)
            };

        PyRunner {
            status: runner.status,
            selection_id: self.id,
            name: runner.name.clone(),
            last_price_traded: runner.last_price_traded,
            total_matched: self.tv.unwrap_or(runner.total_matched),
            adjustment_factor: runner.adjustment_factor,
            handicap: self.hc.or(runner.handicap),
            ex,
            sp,
            sort_priority: runner.sort_priority,
            removal_date: runner.removal_date.clone(),
        }
    }
}
