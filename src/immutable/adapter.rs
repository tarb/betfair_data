use log::warn;
use pyo3::prelude::*;
use pyo3::PyIterProtocol;

use super::iter::ImmutIter;
use crate::errors::IOErr;
use crate::market_source::MarketSource;

#[pyclass]
pub struct ImmutAdapter {
    source: Box<dyn MarketSource + Send>,
}

impl ImmutAdapter {
    pub fn new(source: Box<dyn MarketSource + Send>) -> Self {
        Self { source }
    }
}

#[pyproto]
impl<'p> PyIterProtocol for ImmutAdapter {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        let source_config = slf.source.config();

        loop {
            match slf.source.next() {
                Some(Ok(si)) => {
                    let bflw_iter = ImmutIter::new_object(si, source_config, slf.py());
                    break Some(bflw_iter);
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
