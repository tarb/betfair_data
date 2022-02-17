#![feature(bool_to_option, derive_default_enum, try_blocks)]

mod deser;
mod enums;
mod files_source;
mod ids;
mod market;
mod price_size;
mod runner;
mod strings;
mod tarbz2_source;

use crate::deser::DeserializerWithData;
use crate::market::PyMarket;
use crate::market::PyMarketBase;
use crate::runner::{PyRunner, PyRunnerBookEX, PyRunnerBookSP};
use files_source::Files;
use log::warn;
use price_size::PriceSize;
use pyo3::prelude::*;
use std::path::PathBuf;
use tarbz2_source::TarBz2;

#[cfg(not(target_os = "linux"))]
use mimalloc::MiMalloc;

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct DeserErr {
    pub file: PathBuf,
    pub err: serde_json::Error,
}

pub struct IOErr {
    pub file: Option<PathBuf>,
    pub err: std::io::Error,
}

pub trait MarketSource: pyo3::PyClass {
    fn config(&self) -> SourceConfig;
    fn get(&mut self) -> Option<Result<SourceItem, IOErr>>;

    fn next(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        loop {
            match slf.get() {
                Some(Ok(si)) => {
                    let mi = PyMarket::new_object(si, slf.config(), slf.py());

                    match mi {
                        Ok(mi) => break Some(mi),
                        Err(DeserErr { file, err }) => {
                            warn!(target: "betfair_data", "file: {} err: (JSON Parse Error) {}", file.to_string_lossy(), err);
                        }
                    }
                }
                Some(Err(IOErr {
                    file: Some(name),
                    err,
                })) => {
                    warn!(target: "betfair_data", "file: {} err: (IO Error) {}", name.to_string_lossy(), err);
                }
                Some(Err(IOErr { file: None, err })) => {
                    warn!(target: "betfair_data", "err: (IO Error) {}", err);
                }
                None => break None,
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SourceConfig {
    pub cumulative_runner_tv: bool,
    pub stable_runner_index: bool,
}

pub struct SourceItem {
    pub file: PathBuf,
    pub deser: DeserializerWithData,
}

impl SourceItem {
    pub fn new(file: PathBuf, deser: DeserializerWithData) -> Self {
        Self { file, deser }
    }
}

#[pymodule]
fn betfair_data(_py: Python, m: &PyModule) -> PyResult<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(40)
        .build_global()
        .unwrap();
    pyo3_log::init();

    m.add_class::<Files>()?;
    m.add_class::<TarBz2>()?;
    m.add_class::<PyMarket>()?;
    m.add_class::<PyMarketBase>()?;
    m.add_class::<PyRunner>()?;
    m.add_class::<PyRunnerBookEX>()?;
    m.add_class::<PyRunnerBookSP>()?;
    m.add_class::<PriceSize>()?;

    Ok(())
}
