use core::fmt;
use pyo3::{prelude::*, types::PyList};
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserialize, Deserializer,
};
use std::sync::Arc;

use super::{float_str::FloatStr, runner_book::RunnerBook};
use crate::{
    datetime::DateTimeString, enums::SelectionStatus, ids::SelectionID,
    immutable::container::SyncObj, market_source::SourceConfig, py_rep::PyRep,
};

/*
def __str__(self):
    return "MarketDefinitionRunner: %s" % self.selection_id

def __repr__(self):
    return "<MarketDefinitionRunner>"
*/

#[pyclass]
pub struct MarketDefinitionRunner {
    selection_id: SelectionID,
    #[pyo3(get)]
    adjustment_factor: Option<f64>,
    #[pyo3(get)]
    removal_date: Option<SyncObj<DateTimeString>>,
    #[pyo3(get)]
    sort_priority: u16,
    #[pyo3(get)]
    status: SelectionStatus,
    #[pyo3(get)]
    name: Option<SyncObj<Arc<str>>>,
    #[pyo3(get)]
    bsp: Option<FloatStr>,
}

#[pymethods]
impl MarketDefinitionRunner {
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDefRunnerUpdate<'a> {
    pub id: u32,
    pub adjustment_factor: Option<f64>,
    pub status: SelectionStatus,
    pub sort_priority: u16,
    pub name: Option<&'a str>,
    pub bsp: Option<FloatStr>,
    pub removal_date: Option<&'a str>,
    pub hc: Option<f32>,
}

impl MarketDefinitionRunner {
    fn new(change: &MarketDefRunnerUpdate) -> Self {
        let sid = SelectionID::from((change.id, change.hc));

        Self {
            selection_id: sid,
            status: change.status,
            adjustment_factor: change.adjustment_factor,
            bsp: change.bsp,
            sort_priority: change.sort_priority,
            name: change.name.map(|s| SyncObj::new(Arc::from(s))),
            removal_date: change
                .removal_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
        }
    }

    fn would_change(&self, change: &MarketDefRunnerUpdate) -> bool {
        self.status != change.status
            || self.adjustment_factor != change.adjustment_factor
            || self.sort_priority != change.sort_priority
            || self.bsp != change.bsp
            || ((self.name.is_none() && change.name.is_some())
                || self
                    .name
                    .is_some_and(|s| !change.name.contains(&s.as_ref())))
            || ((self.removal_date.is_none() && change.removal_date.is_some())
                || self
                    .removal_date
                    .is_some_and(|s| !change.removal_date.contains(&s.as_str())))
    }

    fn update_from_change(&self, change: &MarketDefRunnerUpdate) -> Self {
        Self {
            selection_id: self.selection_id,
            status: change.status,
            adjustment_factor: change.adjustment_factor,
            bsp: change.bsp,
            sort_priority: change.sort_priority,
            name: change
                .name
                .and_then(|n| {
                    if self.name.contains(&n) {
                        self.name.clone()
                    } else {
                        Some(SyncObj::new(Arc::from(n)))
                    }
                }),
            removal_date: change
                .removal_date
                .and_then(|n| {
                    if self.removal_date.contains(&n) {
                        self.removal_date.clone()
                    } else {
                        Some(SyncObj::new(DateTimeString::new(n).unwrap()))
                    }
                }),
        }
    }
}

impl PyRep for Vec<Py<MarketDefinitionRunner>> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

pub struct RunnerDefSeq<'a, 'py> {
    pub defs: Option<&'a Vec<Py<MarketDefinitionRunner>>>,
    pub books: Option<&'a Vec<Py<RunnerBook>>>,
    pub py: Python<'py>,
    pub config: SourceConfig,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerDefSeq<'a, 'py> {
    type Value = (
        Option<Vec<Py<MarketDefinitionRunner>>>,
        Option<Vec<Py<RunnerBook>>>,
    );

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            defs: Option<&'a Vec<Py<MarketDefinitionRunner>>>,
            books: Option<&'a Vec<Py<RunnerBook>>>,
            py: Python<'py>,
            #[allow(dead_code)]
            config: SourceConfig,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = (
                Option<Vec<Py<MarketDefinitionRunner>>>,
                Option<Vec<Py<RunnerBook>>>,
            );

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut defs: Option<Vec<Py<MarketDefinitionRunner>>> = None;
                let mut books: Option<Vec<Py<RunnerBook>>> = None;

                while let Some(change) = seq.next_element::<MarketDefRunnerUpdate>()? {
                    let sid = SelectionID::from((change.id, change.hc));

                    let index = {
                        (
                            self.defs.and_then(|rs| {
                                rs.iter()
                                    .map(|r| r.borrow_mut(self.py))
                                    .position(|r| r.selection_id == sid)
                            }),
                            self.books.and_then(|rs| {
                                rs.iter()
                                    .map(|r| r.borrow_mut(self.py))
                                    .position(|r| r.selection_id == sid)
                            }),
                        )
                    };

                    // marketRunnerDef
                    match (self.defs, index.0) {
                        (Some(from), Some(index)) => {
                            let pyr = unsafe { from.get_unchecked(index) };
                            let runner = pyr.borrow(self.py);

                            if runner.would_change(&change) {
                                match defs.as_mut() {
                                    Some(defs) => {
                                        defs.push(
                                            Py::new(self.py, runner.update_from_change(&change))
                                                .unwrap(),
                                        );
                                    }
                                    None => {
                                        defs = {
                                            let mut defs = Vec::with_capacity(std::cmp::min(
                                                from.len() + 1,
                                                10,
                                            ));
                                            defs.push(
                                                Py::new(
                                                    self.py,
                                                    runner.update_from_change(&change),
                                                )
                                                .unwrap(),
                                            );
                                            Some(defs)
                                        };
                                    }
                                };
                            } else {
                                match defs.as_mut() {
                                    Some(defs) => defs.push(pyr.clone_ref(self.py)),
                                    None => {
                                        defs = {
                                            let mut defs = Vec::with_capacity(std::cmp::min(
                                                from.len() + 1,
                                                10,
                                            ));
                                            defs.push(pyr.clone_ref(self.py));
                                            Some(defs)
                                        };
                                    }
                                };
                            }
                        }
                        (Some(from), None) => {
                            let runner =
                                Py::new(self.py, MarketDefinitionRunner::new(&change)).unwrap();

                            match defs.as_mut() {
                                Some(defs) => defs.push(runner),
                                None => {
                                    let mut d =
                                        Vec::with_capacity(std::cmp::max(from.len() + 1, 10));
                                    // from.clone_into(&mut d);

                                    d.push(runner);
                                    defs = Some(d);
                                }
                            };
                        }
                        (None, None) => {
                            let runner =
                                Py::new(self.py, MarketDefinitionRunner::new(&change)).unwrap();

                            match defs.as_mut() {
                                Some(defs) => defs.push(runner),
                                None => {
                                    let mut d = Vec::with_capacity(10);
                                    d.push(runner);
                                    defs = Some(d);
                                }
                            };
                        }
                        _ => unreachable!(),
                    }

                    // runner books
                    match (self.books, index.1) {
                        (Some(from), Some(index)) => {
                            let runner = unsafe { from.get_unchecked(index).borrow(self.py) };

                            if runner.would_change(&change, self.py) {
                                match books.as_mut() {
                                    Some(defs) => {
                                        defs[index] = Py::new(
                                            self.py,
                                            runner.update_from_def(&change, self.py),
                                        )
                                        .unwrap()
                                    }
                                    None => {
                                        books = Some(
                                            from.iter()
                                                .enumerate()
                                                .map(|(i, pr)| {
                                                    if index == i {
                                                        Py::new(
                                                            self.py,
                                                            runner
                                                                .update_from_def(&change, self.py),
                                                        )
                                                        .unwrap()
                                                    } else {
                                                        pr.clone_ref(self.py)
                                                    }
                                                })
                                                .collect(),
                                        );
                                    }
                                };
                            }
                        }
                        (Some(from), None) => {
                            let runner = RunnerBook::new(sid, self.py);
                            let runner =
                                Py::new(self.py, runner.update_from_def(&change, self.py)).unwrap();

                            match books.as_mut() {
                                Some(defs) => defs.push(runner),
                                None => {
                                    let mut d =
                                        Vec::with_capacity(std::cmp::max(from.len() + 1, 10));
                                    from.clone_into(&mut d);

                                    d.push(runner);
                                    books = Some(d);
                                }
                            };
                        }
                        (None, None) => {
                            let runner = RunnerBook::new(sid, self.py);
                            let runner =
                                Py::new(self.py, runner.update_from_def(&change, self.py)).unwrap();

                            match books.as_mut() {
                                Some(defs) => defs.push(runner),
                                None => {
                                    let mut d = Vec::with_capacity(10);
                                    d.push(runner);
                                    books = Some(d);
                                }
                            };
                        }
                        _ => unreachable!(),
                    }
                }

                Ok((defs, books))
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor {
            defs: self.defs,
            books: self.books,
            py: self.py,
            config: self.config,
        })
    }
}
