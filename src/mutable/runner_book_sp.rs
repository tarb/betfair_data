use pyo3::{prelude::*};
use crate::price_size::PriceSize;

#[pyclass(name = "RunnerBookSP")]
#[derive(Default, Clone)]
pub struct PyRunnerBookSP {
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