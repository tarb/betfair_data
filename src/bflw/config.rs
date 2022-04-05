use crate::market_source::ConfigProducer;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub cumulative_runner_tv: bool,
    pub streaming_unique_id: Option<u32>,
}

pub struct ConfigBuilder {
    pub cumulative_runner_tv: bool,
    pub streaming_unique_id: Option<u32>,
}
// clone increments the streaming_unique_id, so that each File has a Config with a unique id
impl ConfigProducer for ConfigBuilder {
    type Config = Config;

    fn get(&mut self) -> Self::Config {
        let c = Config {
            cumulative_runner_tv: self.cumulative_runner_tv,
            streaming_unique_id: self.streaming_unique_id,
        };

        self.streaming_unique_id = self.streaming_unique_id.map(|id| id + 1);
        
        c
    }
}
