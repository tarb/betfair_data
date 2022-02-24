use core::fmt;
use pyo3::{prelude::*, types::PyList};
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserialize, Deserializer,
};

use super::{datetime::DateTimeString, float_str::FloatStr, runner_book::RunnerBook};
use crate::{
    enums::SelectionStatus,
    ids::SelectionID,
    immutable::container::{PyRep, SyncObj},
    market_source::SourceConfig,
};

/*
class MarketDefinitionRunner:
"""
:type adjustment_factor: float
:type id: int
:type removal_date: datetime.datetime
:type sort_priority: int
:type status: unicode
"""

def __init__(
    self,
    id: int,
    sortPriority: int,
    status: str,
    hc: float = 0,
    bsp: float = None,
    adjustmentFactor: float = None,
    removalDate: str = None,
    name: str = None,
):
    self.selection_id = id
    self.sort_priority = sortPriority
    self.status = status
    self.handicap = hc
    self.bsp = bsp
    self.adjustment_factor = adjustmentFactor
    self.removal_date = BaseResource.strip_datetime(removalDate)
    self.name = name  # historic data only

def __str__(self):
    return "MarketDefinitionRunner: %s" % self.selection_id

def __repr__(self):
    return "<MarketDefinitionRunner>"
*/

#[pyclass]
pub struct MarketDefinitionRunner {
    #[pyo3(get)]
    adjustment_factor: Option<f64>,
    #[pyo3(get)]
    selection_id: SelectionID,
    #[pyo3(get)]
    removal_date: Option<SyncObj<DateTimeString>>,
    #[pyo3(get)]
    sort_priority: u16,
    #[pyo3(get)]
    status: SelectionStatus,
    #[pyo3(get)]
    name: Option<SyncObj<String>>,
    #[pyo3(get)]
    handicap: FloatStr,
    #[pyo3(get)]
    bsp: Option<FloatStr>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDefRunnerUpdate<'a> {
    pub id: SelectionID,
    pub adjustment_factor: Option<f64>,
    pub status: SelectionStatus,
    pub sort_priority: u16,
    pub name: Option<&'a str>,
    pub bsp: Option<FloatStr>,
    pub removal_date: Option<&'a str>,
    pub hc: Option<FloatStr>,
}

impl MarketDefinitionRunner {
    fn new(change: &MarketDefRunnerUpdate) -> Self {
        Self {
            selection_id: change.id,
            status: change.status,
            adjustment_factor: change.adjustment_factor,
            handicap: change.hc.unwrap_or(FloatStr(0.0)),
            bsp: change.bsp,
            sort_priority: change.sort_priority,
            name: change.name.map(|s| SyncObj::new(String::from(s))),
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
            || !change.hc.contains(&self.handicap)
            || ((self.name.is_none() && change.name.is_some())
                || self
                    .name
                    .is_some_with(|s| !change.name.contains(&s.value.as_str())))
            || ((self.removal_date.is_none() && change.removal_date.is_some())
                || self
                    .removal_date
                    .is_some_with(|s| !change.removal_date.contains(&s.value.as_str())))
    }

    fn update_from_change(&self, change: &MarketDefRunnerUpdate) -> Self {
        Self {
            selection_id: self.selection_id,
            status: change.status,
            adjustment_factor: change.adjustment_factor.or(self.adjustment_factor),
            handicap: change.hc.unwrap_or(self.handicap),
            bsp: change.bsp.or(self.bsp),
            sort_priority: if self.sort_priority != change.sort_priority {
                change.sort_priority
            } else {
                self.sort_priority
            },

            name: change
                .name
                .and_then(|n| {
                    if self.name.contains(&n) {
                        self.name.clone()
                    } else {
                        Some(SyncObj::new(String::from(n)))
                    }
                })
                .or_else(|| self.name.clone()),

            removal_date: change
                .removal_date
                .and_then(|n| {
                    if self.removal_date.contains(&n) {
                        self.removal_date.clone()
                    } else {
                        Some(SyncObj::new(DateTimeString::new(n).unwrap()))
                    }
                })
                .or_else(|| self.removal_date.clone()),
        }
    }
}

impl PyRep for Vec<Py<MarketDefinitionRunner>> {
    fn py_rep(&self, py: Python) -> PyObject {
        PyList::new(py, self.iter().map(|ps| ps.into_py(py))).into_py(py)
    }
}

pub struct RunnerDefSeq<'a, 'py>(
    pub Option<&'a Vec<Py<MarketDefinitionRunner>>>,
    pub Option<&'a Vec<Py<RunnerBook>>>,
    pub Python<'py>,
    pub SourceConfig,
);
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerDefSeq<'a, 'py> {
    type Value = (
        Option<Vec<Py<MarketDefinitionRunner>>>,
        Option<Vec<Py<RunnerBook>>>,
    );

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py>(
            Option<&'a Vec<Py<MarketDefinitionRunner>>>,
            Option<&'a Vec<Py<RunnerBook>>>,
            Python<'py>,
            SourceConfig,
        );
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
                    let index = {
                        (
                            self.0.and_then(|rs| {
                                rs.iter()
                                    .map(|r| r.borrow_mut(self.2))
                                    .position(|r| r.selection_id == change.id)
                            }),
                            self.1.and_then(|rs| {
                                rs.iter()
                                    .map(|r| r.borrow_mut(self.2))
                                    .position(|r| r.selection_id == change.id)
                            }),
                        )
                    };

                    // NOTE HERE
                    // bflw doesnt reuse the previous ordering of past MarketDefinitionRunners, and the resulting order
                    // should be that of the new runnerdefs

                    // marketRunnerDef
                    match (self.0, index.0) {
                        (Some(from), Some(index)) => {
                            let runner = unsafe { from.get_unchecked(index).borrow(self.2) };

                            if runner.would_change(&change) {
                                match defs.as_mut() {
                                    Some(defs) => {
                                        defs.push(
                                            Py::new(self.2, runner.update_from_change(&change))
                                                .unwrap(),
                                        );
                                        // defs[index] =
                                        //     Py::new(self.2, runner.update_from_change(&change))
                                        //         .unwrap()
                                    }
                                    None => {
                                        defs = {
                                            let mut defs = Vec::with_capacity(std::cmp::min(
                                                from.len() + 1,
                                                10,
                                            ));
                                            defs.push(
                                                Py::new(self.2, runner.update_from_change(&change))
                                                    .unwrap(),
                                            );
                                            Some(defs)
                                        };

                                        // defs = Some(
                                        //     from.iter()
                                        //         .enumerate()
                                        //         .map(|(i, pr)| {
                                        //             if index == i {
                                        //                 Py::new(
                                        //                     self.2,
                                        //                     runner.update_from_change(&change),
                                        //                 )
                                        //                 .unwrap()
                                        //             } else {
                                        //                 pr.clone_ref(self.2)
                                        //             }
                                        //         })
                                        //         .collect(),
                                        // );
                                    }
                                };
                            }
                        }
                        (Some(from), None) => {
                            let runner =
                                Py::new(self.2, MarketDefinitionRunner::new(&change)).unwrap();

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
                                Py::new(self.2, MarketDefinitionRunner::new(&change)).unwrap();

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
                    match (self.1, index.1) {
                        (Some(from), Some(index)) => {
                            let runner = unsafe { from.get_unchecked(index).borrow(self.2) };

                            if runner.would_change(&change, self.2) {
                                match books.as_mut() {
                                    Some(defs) => {
                                        defs[index] =
                                            Py::new(self.2, runner.update_from_def(&change, self.2))
                                                .unwrap()
                                    }
                                    None => {
                                        books = Some(
                                            from.iter()
                                                .enumerate()
                                                .map(|(i, pr)| {
                                                    if index == i {
                                                        Py::new(
                                                            self.2,
                                                            runner.update_from_def(&change, self.2),
                                                        )
                                                        .unwrap()
                                                    } else {
                                                        pr.clone_ref(self.2)
                                                    }
                                                })
                                                .collect(),
                                        );
                                    }
                                };
                            }
                        }
                        (Some(from), None) => {
                            let runner = RunnerBook::new(change.id, self.2);
                            let runner =
                                Py::new(self.2, runner.update_from_def(&change, self.2)).unwrap();

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
                            let runner = RunnerBook::new(change.id, self.2);
                            let runner =
                                Py::new(self.2, runner.update_from_def(&change, self.2)).unwrap();

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

        deserializer.deserialize_seq(RunnerSeqVisitor(self.0, self.1, self.2, self.3))
    }
}
