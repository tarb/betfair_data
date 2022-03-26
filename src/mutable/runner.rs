use pyo3::prelude::*;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::{borrow::Cow, fmt};

use crate::config::Config;
use crate::datetime::DateTimeString;
use crate::enums::SelectionStatus;
use crate::ids::SelectionID;
use crate::mutable::price_size::{PriceSizeBackLadder, PriceSizeLayLadder};
use crate::price_size::F64OrStr;
use crate::py_rep::PyRep;
use crate::strings::{FixedSizeString, StringSetExtNeq};

use super::runner_book_ex::RunnerBookEXMut;
use super::runner_book_sp::RunnerBookSPMut;

#[pyclass(name = "RunnerMut")]
pub struct Runner {
    pub selection_id: SelectionID,
    #[pyo3(get)]
    pub status: SelectionStatus,
    #[pyo3(get)]
    pub name: Option<String>,
    #[pyo3(get)]
    pub last_price_traded: Option<f64>,
    #[pyo3(get)]
    pub total_matched: f64,
    #[pyo3(get)]
    pub adjustment_factor: Option<f64>,
    #[pyo3(get)]
    pub ex: Py<RunnerBookEXMut>,
    #[pyo3(get)]
    pub sp: Py<RunnerBookSPMut>,
    #[pyo3(get)]
    pub sort_priority: u16,
    pub removal_date: Option<DateTimeString>,
}

impl Runner {
    fn new(py: Python) -> Self {
        let ex: RunnerBookEXMut = Default::default();
        let sp: RunnerBookSPMut = Default::default();

        Runner {
            selection_id: Default::default(),
            status: Default::default(),
            name: Default::default(),
            last_price_traded: Default::default(),
            total_matched: Default::default(),
            adjustment_factor: Default::default(),
            sort_priority: Default::default(),
            removal_date: Default::default(),
            ex: Py::new(py, ex).unwrap(),
            sp: Py::new(py, sp).unwrap(),
        }
    }

    pub fn clone(&self, py: Python) -> Self {
        let ex: RunnerBookEXMut = self.ex.borrow(py).clone();
        let sp: RunnerBookSPMut = self.sp.borrow(py).clone();

        Self {
            selection_id: self.selection_id,
            status: self.status,
            name: self.name.clone(),
            last_price_traded: self.last_price_traded,
            total_matched: self.total_matched,
            adjustment_factor: self.adjustment_factor,
            sort_priority: self.sort_priority,
            removal_date: self.removal_date,
            ex: Py::new(py, ex).unwrap(),
            sp: Py::new(py, sp).unwrap(),
        }
    }

    pub fn clear(&mut self, py: Python) {
        self.ex.borrow_mut(py).clear();
        self.sp.borrow_mut(py).clear();
        self.total_matched = 0.0;
        self.last_price_traded = None;
        self.adjustment_factor = None;
    }
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
    #[getter(removal_date)]
    fn get_removeal_date(&self, py: Python) -> PyObject {
        self.removal_date.py_rep(py)
    }
}

pub struct RunnerChangeSeqDeser<'a, 'py> {
    pub runners: &'a mut Vec<Py<Runner>>,
    pub config: Config,
    pub py: Python<'py>,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeSeqDeser<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            runners: &'a mut Vec<Py<Runner>>,
            config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // grow an empty vec
                if self.runners.capacity() == 0 {
                    match seq.size_hint() {
                        Some(s) => self.runners.reserve_exact(s + 2),
                        None => self.runners.reserve_exact(12),
                    }
                }

                #[derive(Deserialize)]
                struct RunnerWithID {
                    id: u32,
                    hc: Option<f32>,
                }

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let parts: RunnerWithID =
                        serde_json::from_str(raw.get()).map_err(Error::custom)?;
                    let rid = SelectionID::from((parts.id, parts.hc));

                    let index = self
                        .runners
                        .iter()
                        .map(|r| r.borrow_mut(self.py))
                        .position(|r| r.selection_id == rid);

                    match index {
                        Some(index) => {
                            let mut runner =
                                unsafe { self.runners.get_unchecked(index).borrow_mut(self.py) };
                            RunnerChangeDeser {
                                runner: &mut runner,
                                config: self.config,
                                py: self.py,
                            }
                            .deserialize(&mut deser)
                            .map_err(Error::custom)?;
                        }
                        None => {
                            let mut runner = Runner::new(self.py);
                            RunnerChangeDeser {
                                runner: &mut runner,
                                config: self.config,
                                py: self.py,
                            }
                            .deserialize(&mut deser)
                            .map_err(Error::custom)?;

                            self.runners.push(Py::new(self.py, runner).unwrap());
                        }
                    }
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor {
            runners: self.runners,
            py: self.py,
            config: self.config,
        })
    }
}

struct RunnerChangeDeser<'a, 'py> {
    pub runner: &'a mut Runner,
    pub config: Config,
    pub py: Python<'py>,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerChangeDeser<'a, 'py> {
    type Value = ();

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
            runner: &'a mut Runner,
            config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerChangeVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Atb => {
                            let mut ex = self.runner.ex.borrow_mut(self.py);
                            let atb = &mut ex.available_to_back;

                            map.next_value_seed(PriceSizeLayLadder(atb))?;
                        }
                        Field::Atl => {
                            let mut ex = self.runner.ex.borrow_mut(self.py);
                            let atl = &mut ex.available_to_lay;

                            map.next_value_seed(PriceSizeBackLadder(atl))?;
                        }
                        Field::Trd => {
                            let mut ex = self.runner.ex.borrow_mut(self.py);
                            let trd = &mut ex.traded_volume;

                            map.next_value_seed(PriceSizeBackLadder(trd))?;

                            if self.config.cumulative_runner_tv {
                                self.runner.total_matched = trd.iter().map(|ps| ps.size).sum();
                            }
                        }
                        Field::Spb => {
                            let mut sp = self.runner.sp.borrow_mut(self.py);
                            let spb = &mut sp.lay_liability_taken;

                            map.next_value_seed(PriceSizeLayLadder(spb))?;
                        }
                        Field::Spl => {
                            let mut sp = self.runner.sp.borrow_mut(self.py);
                            let spl = &mut sp.back_stake_taken;

                            map.next_value_seed(PriceSizeBackLadder(spl))?;
                        }
                        Field::Spn => {
                            let mut sp = self.runner.sp.borrow_mut(self.py);
                            sp.near_price = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Spf => {
                            let mut sp = self.runner.sp.borrow_mut(self.py);
                            sp.far_price = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Ltp => {
                            self.runner.last_price_traded =
                                Some(map.next_value::<F64OrStr>()?.into())
                        }
                        // The betfair historic data files differ from the stream here, they send tv deltas
                        // that need to be accumulated, whereas the stream sends the value itself.
                        Field::Tv => {
                            if self.config.cumulative_runner_tv {
                                map.next_value::<IgnoredAny>()?;
                            } else {
                                self.runner.total_matched = map.next_value::<F64OrStr>()?.into();
                            }
                        }
                        Field::Id => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Hc => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    };
                }

                Ok(())
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
                config: self.config,
                py: self.py,
            },
        )
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunnerDefUpdate<'a> {
    id: u32,
    adjustment_factor: Option<f64>,
    status: SelectionStatus,
    sort_priority: u16,
    name: Option<Cow<'a, str>>,
    bsp: Option<F64OrStr>,
    removal_date: Option<FixedSizeString<24>>,
    hc: Option<f32>,
}

impl<'a> RunnerDefUpdate<'a> {
    fn create(self, py: Python) -> Runner {
        let sp = RunnerBookSPMut {
            actual_sp: self.bsp.map(|f| *f),
            ..Default::default()
        };

        let sid = SelectionID::from((self.id, self.hc));

        Runner {
            selection_id: sid,
            status: self.status,
            adjustment_factor: self.adjustment_factor,
            sort_priority: self.sort_priority,
            name: self.name.map(|s| s.into_owned()),
            removal_date: self
                .removal_date
                .map(|s| DateTimeString::try_from(s).unwrap()),
            sp: Py::new(py, sp).unwrap(),
            ex: Py::new(py, RunnerBookEXMut::default()).unwrap(),
            total_matched: 0.0,
            last_price_traded: None,
        }
    }

    fn update(self, mut runner: PyRefMut<Runner>, py: Python) {
        runner.sp.borrow_mut(py).actual_sp = self.bsp.map(|f| *f);
        runner.adjustment_factor = self.adjustment_factor;
        runner.status = self.status;
        runner.sort_priority = self.sort_priority;

        match self.name {
            Some(s) => {
                runner.name.set_if_ne(s);
            }
            None => {
                runner.name = None;
            }
        }

        match self.removal_date {
            Some(s) if !runner.removal_date.contains(&s) => {
                runner.removal_date = Some(DateTimeString::try_from(s).unwrap());
            }
            None => {
                runner.removal_date = None;
            }
            _ => {}
        }
    }
}

pub struct RunnerDefSeqDeser<'a, 'py> {
    pub runners: &'a mut Vec<Py<Runner>>,
    pub config: Config,
    pub py: Python<'py>,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerDefSeqDeser<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            runners: &'a mut Vec<Py<Runner>>,
            _config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut i = 0;

                while let Some(upt) = seq.next_element::<RunnerDefUpdate>()? {
                    let rid = SelectionID::from((upt.id, upt.hc));

                    let index = self
                        .runners
                        .iter()
                        .map(|r| r.borrow_mut(self.py))
                        .position(|r| r.selection_id == rid);

                    match index {
                        Some(mut index) => {
                            if index != i {
                                self.runners.swap(index, i);
                                index = i;
                            }

                            let runner =
                                unsafe { self.runners.get_unchecked(index).borrow_mut(self.py) };

                            upt.update(runner, self.py);
                        }
                        None => {
                            let runner = upt.create(self.py);

                            self.runners.push(Py::new(self.py, runner).unwrap());
                            let index = self.runners.len() - 1;
                            self.runners.swap(i, index);
                        }
                    }

                    i += 1;
                }

                // remove any runners not found in the runners def,
                // theses will have been swapped to the end of the array
                self.runners.truncate(i);

                Ok(())
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor {
            runners: self.runners,
            _config: self.config,
            py: self.py,
        })
    }
}
