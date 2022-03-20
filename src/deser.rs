use self_cell::self_cell;
use serde_json::{de::StrRead, Deserializer};
use simdutf8::basic::from_utf8;
use std::io::{Error, ErrorKind};

self_cell!(
    pub struct DeserializerWithData {
        owner: Vec<u8>,
        #[covariant]
        dependent: Deser,
    }
);

impl DeserializerWithData {
    pub fn build(bs: Vec<u8>) -> Result<Self, Error> {
        DeserializerWithData::try_new(bs, |bs| {
            let s = from_utf8(bs).map_err(|_| Error::from(ErrorKind::InvalidData))?;
            Ok(Deser(serde_json::Deserializer::from_str(s)))
        })
    }
}

pub struct Deser<'a>(pub Deserializer<StrRead<'a>>);
