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
                        // these errors will be io errors from pulling from the
                        // tar file - not serializations errors of the contained json
                        Some(Err(IOErr {
                            file: Some(name),
                            err,
                        })) => {
                            self.pos = (self.pos + 1) % len;
                            warn!(target: "betfair_data", "source: {} file: {} err: (IO Error) {}", iter.source(), name, err)
                        }
                        Some(Err(IOErr { file: None, err })) => {
                            self.pos = (self.pos + 1) % len;
                            warn!(target: "betfair_data", "source: {} err: (IO Error) {}", iter.source(), err)
                        }

                        // iterator is empty, remove it from the vec
                        // but leave the index the same, as remove shifts
                        // elements to be compact
                        None => {
                            sources.remove(self.pos);
                            self.pos = if len > 1 { self.pos % (len - 1) } else { 0 };
                        }
                    }
                }
                None => break None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{deser::DeserializerWithData, IOErr, MarketSource, SourceItem};

    use super::SourceIter;

    struct TestSource {
        source: &'static str,
        test_count: usize,
        pos: usize,
    }

    impl MarketSource for TestSource {
        fn source(&self) -> &str {
            self.source
        }
    }

    impl Iterator for TestSource {
        type Item = Result<SourceItem, IOErr>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.pos < self.test_count {
                self.pos += 1;
                Some(Ok(SourceItem::new(
                    "source".to_owned(),
                    "file".to_owned(),
                    DeserializerWithData::build(Vec::default()).unwrap(),
                )))
            } else {
                None
            }
        }
    }

    impl TestSource {
        fn new(count: usize) -> Self {
            Self {
                source: "test",
                pos: 0,
                test_count: count,
            }
        }
    }

    #[test]
    fn test_full_iteration() {
        // 1
        let counts: Vec<usize> = vec![111, 127, 1318, 80];
        let sources = SourceIter::new(
            counts
                .iter()
                .map(|c| TestSource::new(*c))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            sources.fold(0, |count, _s| count + 1),
            counts.iter().sum::<usize>(),
        );

        // 2
        let counts: Vec<usize> = vec![1000, 20, 20];
        let sources = SourceIter::new(
            counts
                .iter()
                .map(|c| TestSource::new(*c))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            sources.fold(0, |count, _s| count + 1),
            counts.iter().sum::<usize>(),
        );

        // 3
        let counts: Vec<usize> = vec![10, 1000, 40];
        let sources = SourceIter::new(
            counts
                .iter()
                .map(|c| TestSource::new(*c))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            sources.fold(0, |count, _s| count + 1),
            counts.iter().sum::<usize>(),
        );
    }
}
