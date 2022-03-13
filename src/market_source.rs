use log::warn;
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::deser::DeserializerWithData;
use crate::errors::IOErr;

pub trait MarketSource: Iterator<Item = Result<SourceItem, IOErr>> {
    fn config(&self) -> SourceConfig;
}

#[derive(Debug, Clone, Copy)]
pub struct SourceConfig {
    pub cumulative_runner_tv: bool,
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

pub struct Adapter<T: From<(SourceItem, SourceConfig)>> {
    source: Box<dyn MarketSource + Send>,
    pd: PhantomData<T>,
}

impl<T: From<(SourceItem, SourceConfig)>> Adapter<T> {
    pub fn new(source: Box<dyn MarketSource + Send>) -> Self {
        Self {
            source,
            pd: Default::default(),
        }
    }
}

impl<T: From<(SourceItem, SourceConfig)>> Adapter<T> {
    pub fn next(&mut self) -> Option<T> {
        let source_config = self.source.config();

        loop {
            match self.source.next() {
                Some(Ok(si)) => {
                    let file = T::from((si, source_config));

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
