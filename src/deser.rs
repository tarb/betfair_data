use self_cell::self_cell;
use serde::de::Visitor;
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


impl<'a, 'b> serde::de::Deserializer<'b> for &'a mut Deser<'b>
    where 'a: 'b {

    type Error = serde_json::Error;

    #[inline]
    fn deserialize_any<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_any(visitor)
    }
    #[inline]
    fn deserialize_bool<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_bool(visitor)
    }
    #[inline]
    fn deserialize_i8<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i8(visitor)
    }
    #[inline]
    fn deserialize_i16<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i16(visitor)
    }
    #[inline]
    fn deserialize_i32<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i32(visitor)
    }
    #[inline]
    fn deserialize_i64<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i64(visitor)
    }
    #[inline]
    fn deserialize_u8<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u8(visitor)
    }
    #[inline]
    fn deserialize_u16<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u16(visitor)
    }
    #[inline]
    fn deserialize_u32<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u32(visitor)
    }
    #[inline]
    fn deserialize_u64<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u64(visitor)
    }
    #[inline]
    fn deserialize_f32<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_f32(visitor)
    }
    #[inline]
    fn deserialize_f64<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_f64(visitor)
    }
    #[inline]
    fn deserialize_char<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_char(visitor)
    }
    #[inline]
    fn deserialize_str<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_str(visitor)
    }
    #[inline]
    fn deserialize_string<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_string(visitor)
    }
    #[inline]
    fn deserialize_bytes<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_bytes(visitor)
    }
    #[inline]
    fn deserialize_byte_buf<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_byte_buf(visitor)
    }
    #[inline]
    fn deserialize_option<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_option(visitor)
    }

        #[inline]/// Hint that the `Deserialize` type is expecting a unit value.
    fn deserialize_unit<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_unit(visitor)
    }
    #[inline]
    fn deserialize_unit_struct<V: Visitor<'b>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_unit_struct(name, visitor)
    }
    #[inline]
    fn deserialize_newtype_struct<V: Visitor<'b>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_newtype_struct(name, visitor)
    }
    #[inline]
    fn deserialize_seq<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_seq(visitor)
    }
    #[inline]
    fn deserialize_tuple<V: Visitor<'b>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_tuple(len, visitor)
    }
    #[inline]
    fn deserialize_tuple_struct<V: Visitor<'b>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_tuple_struct(name, len, visitor)
    }
    #[inline]
    fn deserialize_map<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_map(visitor)
    }
    #[inline]
    fn deserialize_struct<V: Visitor<'b>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_struct(name, fields, visitor)
    }
    #[inline]
    fn deserialize_enum<V: Visitor<'b>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_enum(name, variants, visitor)
    }
    #[inline]
    fn deserialize_identifier<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_identifier(visitor)
    }
    #[inline]
    fn deserialize_ignored_any<V: Visitor<'b>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_ignored_any(visitor)
    }
}
