use crate::price_size::PriceSize;
use pyo3::prelude::*;

#[pyclass(name = "RunnerBookSPMut")]
#[derive(Default, Clone)]
pub struct RunnerBookSPMut {
    #[pyo3(get)]
    pub far_price: Option<f64>,
    #[pyo3(get)]
    pub near_price: Option<f64>,
    #[pyo3(get)]
    pub actual_sp: Option<f64>,
    #[pyo3(get)]
    pub back_stake_taken: Vec<PriceSize>,
    #[pyo3(get)]
    pub lay_liability_taken: Vec<PriceSize>,
}

impl RunnerBookSPMut {
    pub fn clear(&mut self) {
        self.back_stake_taken.clear();
        self.lay_liability_taken.clear();
        self.actual_sp = None;
        self.near_price = None;
        self.far_price = None;
    }
}
