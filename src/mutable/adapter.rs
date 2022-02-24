use log::warn;
use pyo3::prelude::*;
use pyo3::PyIterProtocol;

use super::market::PyMarket;
use super::config::Config;
use crate::errors::{DeserErr, IOErr};
use crate::market_source::MarketSource;

#[pyclass]
pub struct MutAdapter {
    source: Box<dyn MarketSource + Send>,
    stable_runner_index: bool,
}

impl MutAdapter {
    pub fn new(source: Box<dyn MarketSource + Send>, stable_runner_index: bool) -> Self {
        Self { source, stable_runner_index }
    }
}

#[pyproto]
impl<'p> PyIterProtocol for MutAdapter {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        let source_config = slf.source.config();
        let config = Config {
            cumulative_runner_tv: source_config.cumulative_runner_tv,
            stable_runner_index: slf.stable_runner_index,
        };

        loop {
            match slf.source.next() {
                Some(Ok(si)) => {
                    let mi = PyMarket::new_object(si, config, slf.py());

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
