use std::path::PathBuf;
use pyo3::prelude::*;

use crate::deser::DeserializerWithData;
use crate::market_source::{SourceConfig, SourceItem};
use super::market_book::MarketBook;


#[pyclass(name="MarketIter")]
pub struct BflwIter {
    file: PathBuf,
    config: SourceConfig,
    deser: Option<DeserializerWithData>,
    state: Option<MarketBook>,
}

impl BflwIter {
    pub fn new_object(
        item: SourceItem,
        config: SourceConfig,
        py: Python,
    ) -> PyObject {

        let iter = BflwIter {
            file: item.file,
            deser: Some(item.deser),
            state: None,
            config,
        };

        iter.into_py(py)
    }

    // fn drive_deserialize(
    //     deser: &mut DeserializerWithData,
    //     base: &mut PyMarketBase,
    //     config: SourceConfig,
    //     py: Python,
    // ) -> Result<(), serde_json::Error> {
    //     deser.with_dependent_mut(|_, deser| {
    //         PyMarketToken(base, py, config).deserialize(&mut deser.0)
    //     })
    // }
}
