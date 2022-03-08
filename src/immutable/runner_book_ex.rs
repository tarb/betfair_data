use std::sync::Arc;
use pyo3::prelude::*;

use crate::immutable::container::SyncObj;
use crate::price_size::PriceSize;

#[derive(Clone, Default)]
#[pyclass]
pub struct RunnerBookEX {
    pub available_to_back: SyncObj<Arc<Vec<PriceSize>>>,
    pub available_to_lay: SyncObj<Arc<Vec<PriceSize>>>,
    pub traded_volume: SyncObj<Arc<Vec<PriceSize>>>,
}

pub struct RunnerBookEXUpdate {
    pub available_to_back: Option<Vec<PriceSize>>,
    pub available_to_lay: Option<Vec<PriceSize>>,
    pub traded_volume: Option<Vec<PriceSize>>,
}

impl RunnerBookEX {
    pub fn update(&self, update: RunnerBookEXUpdate, py: Python) -> Py<Self> {
        Py::new(
            py,
            Self {
                available_to_back: update
                    .available_to_back
                    .map_or_else(|| self.available_to_back.clone(), |ps| SyncObj::new(Arc::new(ps))),
                available_to_lay: update
                    .available_to_lay
                    .map_or_else(|| self.available_to_lay.clone(), |ps| SyncObj::new(Arc::new(ps))),
                traded_volume: update
                    .traded_volume
                    .map_or_else(|| self.traded_volume.clone(), |ps| SyncObj::new(Arc::new(ps))),
            },
        )
        .unwrap()
    }
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
