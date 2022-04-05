use log::warn;
use pyo3::PyClass;
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::deser::DeserializerWithData;
use crate::errors::IOErr;
use crate::files::FilesSource;

pub trait ConfigProducer {
    type Config;

    fn get(&mut self) -> Self::Config;
}

pub struct SourceItem {
    pub file: PathBuf,
    pub deser: DeserializerWithData,
}

impl SourceItem {
    pub fn new(file: PathBuf, deser: DeserializerWithData) -> Self {
        Self { file, deser }
    }
}

pub struct Adapter<C, T> {
    source: FilesSource,
    config: C,
    pd: PhantomData<T>,
}

impl<C, T> Adapter<C, T>
where
    C: ConfigProducer,
    C::Config: Copy + Clone,
    T: From<(SourceItem, C::Config)> + PyClass,
{
    pub fn new(source: FilesSource, config: C) -> Self {
        Self {
            source,
            config,
            pd: Default::default(),
        }
    }
}

impl<C, T> Adapter<C, T>
where
    C: ConfigProducer,
    C::Config: Copy + Clone,
    T: From<(SourceItem, C::Config)> + PyClass,
{
    pub fn next(&mut self) -> Option<T> {
        loop {
            match self.source.next() {
                Some(Ok(si)) => {
                    let file = T::from((si, self.config.get()));

                    break Some(file);
                }
                Some(Err(IOErr {
                    file: Some(name),
                    err,
                })) => {
                    warn!(target: "betfair_data", "file: {} err: (IO Error) {}", name.to_string_lossy(), err);
                }
                Some(Err(IOErr { file: None, err })) => {
                    warn!(target: "betfair_data", "err: (IO Error) {}", err);
                }
                None => break None,
            }
        }
    }
}
