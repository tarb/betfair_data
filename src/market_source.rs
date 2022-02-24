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
