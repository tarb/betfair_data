#![feature(bool_to_option, derive_default_enum)]

mod deser;
mod enums;
mod ids;
mod market;
mod price_size;
mod runner;
mod source_iter;
mod strings;
mod tarbz2_source;

use deser::DeserializerWithData;
use log::warn;
use market::PyMarketBase;
use price_size::PriceSize;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PySequence, PyString};
use pyo3::PyIterProtocol;
use source_iter::SourceIter;
use tarbz2_source::TarBzSource;

#[cfg(not(target_os = "linux"))]
use mimalloc::MiMalloc;

use crate::market::PyMarket;
use crate::runner::{PyRunner, PyRunnerBookEX, PyRunnerBookSP};

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct DeserErr {
    pub source: String,
    pub file: String,
    pub err: serde_json::Error,
}

pub struct IOErr {
    pub file: Option<String>,
    pub err: std::io::Error,
}

pub trait MarketSource: Iterator<Item = Result<SourceItem, IOErr>> + Send {
    fn source(&self) -> &str;
}

#[derive(Debug, Clone, Copy)]
pub struct SourceConfig {
    pub cumulative_runner_tv: bool,
    pub stable_runner_index: bool,
}

pub struct SourceItem {
    pub source: String,
    pub file: String,
    // pub bs: Vec<u8>,
    pub deser: DeserializerWithData,
}

impl SourceItem {
    pub fn new(source: String, file: String, deser: DeserializerWithData) -> Self {
        Self {
            source,
            file,
            deser,
        }
    }
}

#[pyclass]
struct TarBz2 {
    sources: SourceIter<TarBzSource>,
    config: SourceConfig,
}

#[pymethods]
impl TarBz2 {
    #[new]
    #[args(cumulative_runner_tv = "true", stable_runner_index = "true")]
    fn __new__(
        paths: &PySequence,
        cumulative_runner_tv: bool,
        stable_runner_index: bool,
    ) -> PyResult<Self> {
        let config = SourceConfig {
            cumulative_runner_tv,
            stable_runner_index,
        };
        // TODO: one day PySequence might implementer iter, like PyTuple and PyList
        let sources = (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .map(|s| TarBzSource::new(s?))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|op: std::io::Error| {
                PyErr::new::<exceptions::PyRuntimeError, _>(op.to_string())
            })?;

        Ok(Self {
            config,
            sources: SourceIter::new(sources),
        })
    }
}

#[pyproto]
impl<'p> PyIterProtocol for TarBz2 {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        loop {
            match slf.sources.next() {
                Some(si) => {
                    let mi = PyMarket::new_object(si, slf.config, slf.py());

                    match mi {
                        Ok(mi) => break Some(mi),
                        Err(DeserErr { source, file, err }) => {
                            warn!(target: "betfair_data", "source: {} file: {} err: (JSON Parse Error) {}", source, file, err);
                        }
                    }
                }
                None => break None,
            }
        }
    }
}

#[pymodule]
fn betfair_data(_py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<TarBz2>()?;
    m.add_class::<PyMarket>()?;
    m.add_class::<PyMarketBase>()?;
    m.add_class::<PyRunner>()?;
    m.add_class::<PyRunnerBookEX>()?;
    m.add_class::<PyRunnerBookSP>()?;
    m.add_class::<PriceSize>()?;

    Ok(())
}
