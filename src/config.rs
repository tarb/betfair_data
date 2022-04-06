use crate::market_source::ConfigProducer;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub cumulative_runner_tv: bool,
}

impl ConfigProducer for Config {
    type Config = Self;

    #[inline]
    fn get(&mut self) -> Self::Config {
        *self
    }
}