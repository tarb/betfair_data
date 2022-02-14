use crossbeam_channel::{bounded, Receiver};
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::thread;
use tar::Archive;
// use bzip2_rs::decoder::DecoderReader;
use bzip2_rs::decoder::ParallelDecoderReader;

use crate::deser::DeserializerWithData;
use crate::{IOErr, MarketSource, SourceItem};

pub struct TarBzSource {
    source: String,
    chan: Receiver<Result<TarEntry, (Error, Option<String>)>>,
}

struct TarEntry {
    name: String,
    // bs: Vec<u8>,
    deser: DeserializerWithData,
}

impl MarketSource for TarBzSource {
    fn source(&self) -> &str {
        &self.source
    }
}

impl TarBzSource {
    pub fn new<S: Into<String>>(path: S) -> Result<Self, Error> {
        let path = path.into();
        let (data_send, data_recv) = bounded(5);
        let file = File::open(&path)?;

        // on TarBzSource drop, data_send will become disconnected, causing send to fail with an error
        // this will cause try_for_each to fail and the iter&thread to stop and close
        thread::spawn(move || -> Result<(), Error> {
            let _ = Archive::new(file)
                .entries()?
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| (entry.size() > 0).then_some(entry))
                .map(|entry| {
                    let mut buf = Vec::with_capacity(entry.size() as usize);
                    let name = String::from_utf8_lossy(entry.path_bytes().as_ref()).into_owned();

                    // DecoderReader::new(entry)
                    let r =
                        ParallelDecoderReader::new(entry, bzip2_rs::RayonThreadPool, 1024 * 1024)
                            .read_to_end(&mut buf)
                            .and_then(|_| DeserializerWithData::build(buf));

                    match r {
                        Ok(deser) => Ok(TarEntry { name, deser }),
                        Err(err) => Err((err, Some(name))),
                    }
                })
                .try_for_each(|data| data_send.send(data));

            Ok(())
        });

        Ok(Self {
            source: path,
            chan: data_recv,
        })
    }
}

impl Iterator for TarBzSource {
    type Item = Result<SourceItem, IOErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chan.recv().ok().map(|r| match r {
            Ok(entry) => Ok(SourceItem::new(
                self.source.clone(),
                entry.name,
                entry.deser,
            )),
            Err((err, name)) => Err(IOErr { file: name, err }),
        })
    }
}
