use crate::price_size::PriceSize;
use pyo3::prelude::*;

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

impl RunnerBookEXMut {
    pub fn clear(&mut self) {
        self.available_to_back.clear();
        self.available_to_lay.clear();
        self.traded_volume.clear();
    }
}
