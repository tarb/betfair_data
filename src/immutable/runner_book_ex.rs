use pyo3::prelude::*;
use std::sync::Arc;

use crate::immutable::container::SyncObj;
use crate::price_size::PriceSize;

#[derive(Clone, Default)]
#[pyclass]
pub struct RunnerBookEX {
    pub available_to_back: SyncObj<Arc<Vec<PriceSize>>>,
    pub available_to_lay: SyncObj<Arc<Vec<PriceSize>>>,
    pub traded_volume: SyncObj<Arc<Vec<PriceSize>>>,
}

#[pymethods]
impl RunnerBookEX {
    #[getter(available_to_back)]
    fn get_available_to_back(&self, py: Python) -> PyObject {
        self.available_to_back.to_object(py)
    }
    #[getter(available_to_lay)]
    fn get_available_to_lay(&self, py: Python) -> PyObject {
        self.available_to_lay.to_object(py)
    }
    #[getter(traded_volume)]
    fn get_traded_volume(&self, py: Python) -> PyObject {
        self.traded_volume.to_object(py)
    }
}
