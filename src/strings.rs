use core::fmt;
use std::{array::TryFromSliceError, str};

use serde::{de::Error, de::Visitor, Deserialize, Deserializer};

pub trait StringSetExtNeq {
    fn set_if_ne<S: Into<String> + AsRef<str>>(&mut self, s: S) -> bool;
}

impl StringSetExtNeq for String {
    fn set_if_ne<S: Into<String> + AsRef<str>>(&mut self, s: S) -> bool {
        if s.as_ref() != self {
            self.clear();
            self.push_str(s.as_ref());
            true
        } else {
            false
        }
    }
}

impl StringSetExtNeq for Option<String> {
    fn set_if_ne<S: Into<String> + AsRef<str>>(&mut self, s: S) -> bool {
        match self {
            Some(str) => str.set_if_ne(s),
            None => {
                *self = Some(s.into());
                true
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FixedSizeString<const N: usize>([u8; N]);

impl<const N: usize> FixedSizeString<N> {
    #[inline]
    pub fn new(s: &str) -> Result<FixedSizeString<N>, TryFromSliceError> {
        let a: [u8; N] = s.as_bytes().try_into()?;
        Ok(Self(a))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> TryFrom<&str> for FixedSizeString<N> {
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl<const N: usize> Default for FixedSizeString<N> {
    fn default() -> Self {
        let a: [u8; N] = [0; N];
        Self(a)
    }
}

impl<const N: usize> AsRef<str> for FixedSizeString<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

// should be always safe, as its always made from a str which has been checked to be valid bytes
impl<const N: usize> std::ops::Deref for FixedSizeString<N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<const N: usize> fmt::Display for FixedSizeString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl<const N: usize> fmt::Debug for FixedSizeString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl<const N: usize> str::FromStr for FixedSizeString<N> {
    type Err = TryFromSliceError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<const N: usize> PartialEq<FixedSizeString<N>> for &str {
    #[inline]
    fn eq(&self, s: &FixedSizeString<N>) -> bool {
        *self == s.as_ref()
    }
}

impl<'de, const N: usize> Deserialize<'de> for FixedSizeString<N> {
    fn deserialize<D>(deserializer: D) -> Result<FixedSizeString<N>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FSSVisitor<const L: usize>;

        impl<'de, const L: usize> Visitor<'de> for FSSVisitor<L> {
            type Value = FixedSizeString<L>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing \"xxx\"")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                FixedSizeString::<L>::new(s).map_err(Error::custom)
            }
        }

        deserializer
            .deserialize_str(FSSVisitor)
            .map_err(Error::custom)
    }
}
