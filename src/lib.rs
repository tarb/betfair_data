#![feature(
    bool_to_option,
    derive_default_enum
)]

mod market;
mod runner;
mod price_size;
mod tarbz2_source;
mod deser;
mod enums;
mod ids;
mod strings;

use log::{warn, info};
use market::PyMarketBase;
use price_size::PriceSize;
use tarbz2_source::TarBzSource;
use pyo3::exceptions;
use pyo3::prelude::*; 
use pyo3::PyIterProtocol;
use pyo3::types::{PyString, PySequence};
use pyo3_log;

use crate::market::PyMarket;
use crate::runner::{PyRunner, PyRunnerBookEX, PyRunnerBookSP};


pub struct DeserErr {
    pub source: String,
    pub file: String,
    pub err: serde_json::Error,
}

pub struct IOErr {
    pub file: Option<String>,
    pub err: std::io::Error,
}

trait MarketSource: Iterator<Item = SourceItem> {
    fn source(&self) -> &str;
}

#[derive(Debug)]
pub struct SourceItem {
    pub source: String,
    pub file: String,
    pub bs: Vec<u8>,
}

impl SourceItem {
    pub fn new(source: String, file: String, bs: Vec<u8>) -> Self {
        Self { 
            source,
            file,
            bs,
        }
    }
}


#[pyclass]
struct Sources {
    // sources: std::iter::Flatten<IntoIter<TarBzSource>>
    sources: Vec<TarBzSource>,
    pos: usize,
}

impl Iterator for Sources {
    type Item = SourceItem;
    
    fn next(&mut self) -> Option<Self::Item> {
        let sources = &mut self.sources;

        loop {
            let len = sources.len();
 
            match sources.get_mut(self.pos) {
                Some(iter) => {
                    match iter.next() {
                        // iterator had good value, progress the iter and increment
                        // the index wrapping length if needed
                        Some(Ok(si)) => {
                            self.pos = (self.pos + 1) % len;
                            info!(target: "betfair_data", "source: {} file: {}", si.source, si.file);
                            break Some(si);
                        },
                        // iterator contained a value, but that value was an error
                        // these errors will be io erros from pulling from the
                        // tar file - not serializations errors of the contained json
                        Some(Err(IOErr{ file: Some(name), err } )) => 
                            warn!(target: "betfair_data", "source: {} file: {} err: (IO Error) {}", iter.source, name, err),
                        Some(Err(IOErr{ file: None, err } ))  => 
                            warn!(target: "betfair_data", "source: {} err: (IO Error) {}", iter.source, err),

                        // iterator is empty, remove it from the vec
                        // but leave the index the same, as remove shifts
                        // elements to be compact
                        None => { sources.remove(self.pos); },
                    }
                }
                None => break None,
            }
        }
    }
}




#[pyproto]
impl<'p> PyIterProtocol for Sources {
    fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
        loop {
            match slf.next() {
                Some(si) => {
                    let mi = PyMarket::new_object(si, slf.py());
        
                    match mi {
                        Ok(mi) => break Some(mi),
                        Err(DeserErr { source, file, err }) => {
                            warn!(target: "betfair_data", "source: {} file: {} err: (JSON Parse Error) {}", source, file, err);
                        }
                    }
                }
                None => break None,
            }
        }
    }

}

#[pymethods]
impl Sources {

    #[new]
    fn __new__(paths: &PySequence) -> PyResult<Self> {
        // TODO: one day PySequence might implementer iter, like PyTuple and PyList
        let sources = (0..paths.len().unwrap_or(0))
            .filter_map(|index| paths.get_item(index).ok())
            .filter_map(|any| any.downcast::<PyString>().map(|ps| ps.to_str()).ok())
            .map(|s| TarBzSource::new(s?))
            .collect::<Result<Vec<_>, _>>().map_err(|op| PyErr::new::<exceptions::PyRuntimeError, _>(op.to_string()))?;

        Ok(Self { sources, pos: 0 })
    }

}

#[pymodule]
fn betfair_data(_py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<Sources>()?;
    m.add_class::<PyMarket>()?;
    m.add_class::<PyMarketBase>()?;
    m.add_class::<PyRunner>()?;
    m.add_class::<PyRunnerBookEX>()?;
    m.add_class::<PyRunnerBookSP>()?;
    m.add_class::<PriceSize>()?;

    Ok(())
}
