use std::sync::Arc;

use pyo3::prelude::*;

use super::container::SyncObj;
use crate::price_size::PriceSize;

#[derive(Clone, Default)]
#[pyclass]
pub struct RunnerBookSP {
    #[pyo3(get)]
    pub actual_sp: Option<f64>,
    #[pyo3(get)]
    pub far_price: Option<f64>,
    #[pyo3(get)]
    pub near_price: Option<f64>,
    pub back_stake_taken: SyncObj<Arc<Vec<PriceSize>>>,
    pub lay_liability_taken: SyncObj<Arc<Vec<PriceSize>>>,
}

#[pymethods]
impl RunnerBookSP {
    #[getter(back_stake_taken)]
    fn get_back_stake_taken(&self, py: Python) -> PyObject {
        self.back_stake_taken.to_object(py)
    }
    #[getter(lay_liability_taken)]
    fn get_lay_liability_taken(&self, py: Python) -> PyObject {
        self.lay_liability_taken.to_object(py)
    }
}
