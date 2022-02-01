use self_cell::self_cell;
use serde_json::{de::SliceRead, Deserializer};

pub struct Deser<'a>(pub Deserializer<SliceRead<'a>>);

self_cell!(
    pub struct DeserializerWithData {
        owner: Vec<u8>,
        #[covariant]
        dependent: Deser,
    }
);
