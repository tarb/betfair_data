use log::warn;
use pyo3::prelude::*;
use std::path::PathBuf;

use crate::deser::DeserializerWithData;
use crate::errors::DeserErr;
use crate::errors::IOErr;
use crate::mutable::market::PyMarket;

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