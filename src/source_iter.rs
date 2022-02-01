use log::{info, warn};

use crate::IOErr;
use crate::MarketSource;
use crate::SourceItem;

pub struct SourceIter<T: MarketSource> {
    sources: Vec<T>,
    pos: usize,
}

impl<T: MarketSource> SourceIter<T> {
    pub fn new(sources: Vec<T>) -> Self {
        Self { sources, pos: 0 }
    }
}

impl<T: MarketSource> Iterator for SourceIter<T> {
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
                        }
                        // iterator contained a value, but that value was an error
                        // these errors will be io erros from pulling from the
                        // tar file - not serializations errors of the contained json
                        Some(Err(IOErr {
                            file: Some(name),
                            err,
                        })) => {
                            warn!(target: "betfair_data", "source: {} file: {} err: (IO Error) {}", iter.source(), name, err)
                        }
                        Some(Err(IOErr { file: None, err })) => {
                            warn!(target: "betfair_data", "source: {} err: (IO Error) {}", iter.source(), err)
                        }

                        // iterator is empty, remove it from the vec
                        // but leave the index the same, as remove shifts
                        // elements to be compact
                        None => {
                            sources.remove(self.pos);
                        }
                    }
                }
                None => break None,
            }
        }
    }
}