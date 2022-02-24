use pyo3::prelude::*;

use crate::immutable::container::SyncObj;
use crate::price_size::PriceSize;

use super::float_str::FloatStr;

#[derive(Clone, Default)]
#[pyclass]
pub struct RunnerBookSP {
    #[pyo3(get)]
    pub actual_sp: Option<FloatStr>,
    #[pyo3(get)]
    pub far_price: Option<FloatStr>,
    #[pyo3(get)]
    pub near_price: Option<FloatStr>,
    pub back_stake_taken: SyncObj<Vec<PriceSize>>,
    pub lay_liability_taken: SyncObj<Vec<PriceSize>>,
}

#[derive(Default)]
pub struct RunnerBookSPUpdate {
    pub actual_sp: Option<FloatStr>,
    pub far_price: Option<FloatStr>,
    pub near_price: Option<FloatStr>,
    pub back_stake_taken: Option<Vec<PriceSize>>,
    pub lay_liability_taken: Option<Vec<PriceSize>>,
}

impl RunnerBookSP {
    pub fn update(&self, update: RunnerBookSPUpdate, py: Python) -> Py<Self> {
        Py::new(
            py,
            Self {
                actual_sp: update.actual_sp.or(self.actual_sp),
                far_price: update.far_price.or(self.far_price),
                near_price: update.near_price.or(self.near_price),
                back_stake_taken: update
                    .back_stake_taken
                    .map_or_else(|| self.back_stake_taken.clone(), SyncObj::new),
                lay_liability_taken: update
                    .lay_liability_taken
                    .map_or_else(|| self.lay_liability_taken.clone(), SyncObj::new),
            },
        )
        .unwrap()
    }
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
