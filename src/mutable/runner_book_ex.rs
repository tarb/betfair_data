use pyo3::{prelude::*};
use crate::price_size::PriceSize;

#[pyclass(name = "RunnerBookEXMut")]
#[derive(Default, Clone)]
pub struct RunnerBookEXMut {
    #[pyo3(get)]
    pub available_to_back: Vec<PriceSize>,
    #[pyo3(get)]
    pub available_to_lay: Vec<PriceSize>,
    #[pyo3(get)]
    pub traded_volume: Vec<PriceSize>,
}
