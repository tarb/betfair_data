use pyo3::prelude::*;
use std::fmt;
// use pyo3::types::PyDateTime;
use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;
use staticvec::StaticString;

use crate::ids::SelectionID;
use crate::price_size::{F64OrStr, PriceSize, PriceSizeBackLadder, PriceSizeLayLadder};
use crate::strings::StringSetExtNeq;
use crate::{enums::SelectionStatus, SourceConfig};

#[pyclass(name = "Runner")]
pub struct PyRunner {
    #[pyo3(get)]
    pub selection_id: SelectionID,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub last_price_traded: Option<f64>,
    #[pyo3(get)]
    pub total_volume: f64,
    #[pyo3(get)]
    pub adjustment_factor: Option<f64>,
    #[pyo3(get)]
    pub handicap: Option<f64>,
    #[pyo3(get)]
    pub ex: Py<PyRunnerBookEX>,
    #[pyo3(get)]
    pub sp: Py<PyRunnerBookSP>,
    #[pyo3(get)]
    pub sort_priority: u16,
    #[pyo3(get)]
    pub removal_date: Option<i64>,
    // removal_date: Option<Py<PyDateTime>>,
    pub removal_date_str: Option<StaticString<24>>,

    // requires a getter
    pub status: SelectionStatus,
}

impl PyRunner {
    fn new(py: Python) -> Self {
        let ex: PyRunnerBookEX = Default::default();
        let sp: PyRunnerBookSP = Default::default();

        PyRunner {
            selection_id: Default::default(),
            status: Default::default(),
            name: Default::default(),
            last_price_traded: Default::default(),
            total_volume: Default::default(),
            adjustment_factor: Default::default(),
            handicap: Default::default(),
            sort_priority: Default::default(),
            removal_date_str: Default::default(),
            removal_date: Default::default(),
            ex: Py::new(py, ex).unwrap(),
            sp: Py::new(py, sp).unwrap(),
        }
    }

    pub fn clone(&self, py: Python) -> Self {
        let ex: PyRunnerBookEX = self.ex.borrow(py).clone();
        let sp: PyRunnerBookSP = self.sp.borrow(py).clone();

        Self {
            selection_id: self.selection_id,
            status: self.status,
            name: self.name.clone(),
            last_price_traded: self.last_price_traded,
            total_volume: self.total_volume,
            adjustment_factor: self.adjustment_factor,
            handicap: self.handicap,
            sort_priority: self.sort_priority,
            removal_date_str: self.removal_date_str.clone(),
            removal_date: self.removal_date.clone(),
            ex: Py::new(py, ex).unwrap(),
            sp: Py::new(py, sp).unwrap(),
        }
    }
}

#[pymethods]
impl PyRunner {
    #[getter(status)]
    fn status(&self) -> &'static str {
        self.status.into()
    }
}

#[pyclass(name = "RunnerBookEX")]
#[derive(Default, Clone)]
pub struct PyRunnerBookEX {
    #[pyo3(get)]
    available_to_back: Vec<PriceSize>,
    #[pyo3(get)]
    available_to_lay: Vec<PriceSize>,
    #[pyo3(get)]
    traded_volume: Vec<PriceSize>,
}

#[pyclass(name = "RunnerBookSP")]
#[derive(Default, Clone)]
pub struct PyRunnerBookSP {
    #[pyo3(get)]
    far_price: Option<f64>,
    #[pyo3(get)]
    near_price: Option<f64>,
    #[pyo3(get)]
    actual_sp: Option<f64>,
    #[pyo3(get)]
    back_stake_taken: Vec<PriceSize>,
    #[pyo3(get)]
    lay_liability_taken: Vec<PriceSize>,
}

pub struct PyRunnerDefSeq<'a, 'py>(
    pub &'a mut Vec<Py<PyRunner>>,
    pub Python<'py>,
    pub SourceConfig,
);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerDefSeq<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py>(&'a mut Vec<Py<PyRunner>>, Python<'py>, SourceConfig);
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
                if self.0.capacity() == 0 {
                    match seq.size_hint() {
                        Some(s) => self.0.reserve_exact(s + 2),
                        None => self.0.reserve_exact(12),
                    }
                }

                // WARNING
                // So this grossness, is due to us needing to get the selection_id to find the
                // runner to deserialize in place into. The id is not the first element of the
                // object, so we'd need to defer parsing the other properties, then come back
                // to them once we know the correct selection to use as the seed.
                // What we do here is parse the json twice, once pulling out only the id, then
                // again as normal
                #[derive(Deserialize)]
                struct RunnerWithID {
                    id: SelectionID,
                }

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let rid: RunnerWithID = serde_json::from_str(raw.get()).unwrap(); //TODO: fix unwrap later

                    let index = self
                        .0
                        .iter()
                        .map(|r| r.borrow_mut(self.1))
                        .position(|r| r.selection_id == rid.id);
                    match index {
                        Some(index) => {
                            let mut runner =
                                unsafe { self.0.get_unchecked(index).borrow_mut(self.1) };
                            PyRunnerDefinitonDeser(&mut runner, self.1, self.2)
                                .deserialize(&mut deser)
                                .unwrap(); // TODO: fix unwrap later;
                        }
                        None => {
                            let mut runner = PyRunner::new(self.1);
                            PyRunnerDefinitonDeser(&mut runner, self.1, self.2)
                                .deserialize(&mut deser)
                                .unwrap(); // TODO: fix unwrap later;

                            self.0.push(Py::new(self.1, runner).unwrap());
                        }
                    }
                }

                // this config flag will reorder the runners into the order specified in the sort priority
                // as seen in the data files
                if self.2.stable_runner_index == false {
                    self.0.sort_by_key(|r| r.borrow(self.1).sort_priority);
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor(self.0, self.1, self.2))
    }
}

struct PyRunnerDefinitonDeser<'a, 'py>(&'a mut PyRunner, Python<'py>, SourceConfig);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerDefinitonDeser<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            AdjustmentFactor,
            Status,
            SortPriority,
            Id,
            Name,
            Bsp,
            RemovalDate,
            Hc
        }

        struct RunnerDefVisitor<'a, 'py>(&'a mut PyRunner, Python<'py>, SourceConfig);
        impl<'de, 'a, 'py> Visitor<'de> for RunnerDefVisitor<'a, 'py> {
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
                        Field::Id => self.0.selection_id = map.next_value()?,
                        Field::AdjustmentFactor => {
                            self.0.adjustment_factor = Some(map.next_value::<F64OrStr>()?.into())
                        }
                        Field::Status => self.0.status = map.next_value()?,
                        Field::SortPriority => self.0.sort_priority = map.next_value()?,
                        Field::Hc => self.0.handicap = Some(map.next_value::<F64OrStr>()?.into()),
                        Field::Name => {
                            self.0.name.set_if_ne(map.next_value()?);
                        }
                        Field::RemovalDate => {
                            let s = map.next_value()?;
                            if self.0.removal_date_str.set_if_ne(s) {
                                let ts = chrono::DateTime::parse_from_rfc3339(s)
                                    .unwrap()
                                    .timestamp_millis()
                                    / 1000;
                                // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
                                // self.0.removal_date = Some(d.into_py(self.1));
                                self.0.removal_date = Some(ts);
                            }
                        }
                        Field::Bsp => {
                            let mut sp = self.0.sp.borrow_mut(self.1);
                            sp.actual_sp = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        
                    }
                }

                Ok(())
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "adjustmentFactor",
            "status",
            "sortPriority",
            "id",
            "name",
            "bsp",
            "removalDate",
        ];
        deserializer.deserialize_struct(
            "RunnerDef",
            FIELDS,
            RunnerDefVisitor(self.0, self.1, self.2),
        )
    }
}

pub struct PyRunnerChangeSeq<'a, 'py>(
    pub &'a mut Vec<Py<PyRunner>>,
    pub Python<'py>,
    pub SourceConfig,
);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerChangeSeq<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py>(&'a mut Vec<Py<PyRunner>>, Python<'py>, SourceConfig);
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
                if self.0.capacity() == 0 {
                    match seq.size_hint() {
                        Some(s) => self.0.reserve_exact(s + 2),
                        None => self.0.reserve_exact(12),
                    }
                }

                #[derive(Deserialize)]
                struct RunnerWithID {
                    id: SelectionID,
                }

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deser = serde_json::Deserializer::from_str(raw.get());
                    let rid: RunnerWithID = serde_json::from_str(raw.get()).unwrap(); //TODO: fix unwrap later

                    let index = self
                        .0
                        .iter()
                        .map(|r| r.borrow_mut(self.1))
                        .position(|r| r.selection_id == rid.id);
                    match index {
                        Some(index) => {
                            let mut runner =
                                unsafe { self.0.get_unchecked(index).borrow_mut(self.1) };
                            PyRunnerChangeDeser(&mut runner, self.1, self.2)
                                .deserialize(&mut deser)
                                .unwrap(); // TODO: fix unwrap later;
                        }
                        None => {
                            let mut runner = PyRunner::new(self.1);
                            PyRunnerChangeDeser(&mut runner, self.1, self.2)
                                .deserialize(&mut deser)
                                .unwrap(); // TODO: fix unwrap later;

                            self.0.push(Py::new(self.1, runner).unwrap());
                        }
                    }
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor(self.0, self.1, self.2))
    }
}

struct PyRunnerChangeDeser<'a, 'py>(&'a mut PyRunner, Python<'py>, SourceConfig);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyRunnerChangeDeser<'a, 'py> {
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

        struct RunnerChangeVisitor<'a, 'py>(&'a mut PyRunner, Python<'py>, SourceConfig);
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
                        Field::Id => self.0.selection_id = map.next_value()?,
                        Field::Atb => {
                            let mut ex = self.0.ex.borrow_mut(self.1);
                            map.next_value_seed(PriceSizeBackLadder(&mut ex.available_to_back))?;
                        }
                        Field::Atl => {
                            let mut ex = self.0.ex.borrow_mut(self.1);
                            map.next_value_seed(PriceSizeLayLadder(&mut ex.available_to_lay))?;
                        }
                        Field::Trd => {
                            let mut ex = self.0.ex.borrow_mut(self.1);
                            map.next_value_seed(PriceSizeBackLadder(&mut ex.traded_volume))?;
                        }
                        Field::Spb => {
                            let mut sp = self.0.sp.borrow_mut(self.1);
                            map.next_value_seed(PriceSizeBackLadder(&mut sp.lay_liability_taken))?;
                        }
                        Field::Spl => {
                            let mut sp = self.0.sp.borrow_mut(self.1);
                            map.next_value_seed(PriceSizeLayLadder(&mut sp.back_stake_taken))?;
                        }
                        Field::Spn => {
                            let mut sp = self.0.sp.borrow_mut(self.1);
                            sp.near_price = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Spf => {
                            let mut sp = self.0.sp.borrow_mut(self.1);
                            sp.far_price = Some(map.next_value::<F64OrStr>()?.into());
                        }
                        Field::Ltp => {
                            self.0.last_price_traded = Some(map.next_value::<F64OrStr>()?.into())
                        }
                        Field::Hc => self.0.handicap = Some(map.next_value::<F64OrStr>()?.into()),
                        // The betfair historic data files differ from the stream here, they send tv deltas
                        // that need to be accumulated, whereas the stream sends the value itself.
                        Field::Tv => {
                            if self.2.cumulative_runner_tv {
                                self.0.total_volume = {
                                    let delta: f64 = map.next_value::<F64OrStr>()?.into();
                                    self.0.total_volume + delta
                                };
                            } else {
                                self.0.total_volume = map.next_value::<F64OrStr>()?.into();
                            }
                        }
                    };
                }

                Ok(())
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "id", "atb", "atl", "spn", "spf", "spb", "spl", "trd", "tv", "ltp", "hc",
        ];
        deserializer.deserialize_struct(
            "RunnerChange",
            FIELDS,
            RunnerChangeVisitor(self.0, self.1, self.2),
        )
    }
}
