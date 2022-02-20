use staticvec::StaticString;

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

impl<const N: usize> StringSetExtNeq for StaticString<N> {
    fn set_if_ne<S: Into<String> + AsRef<str>>(&mut self, s: S) -> bool {
        if s.as_ref() != self.as_str() {
            self.clear();
            self.push_str(s);
            true
        } else {
            false
        }
    }
}

impl<const N: usize> StringSetExtNeq for Option<StaticString<N>> {
    fn set_if_ne<S: Into<String> + AsRef<str>>(&mut self, s: S) -> bool {
        match self {
            Some(str) => str.set_if_ne(s),
            None => {
                *self = Some(StaticString::from(s.as_ref()));
                true
            }
        }
    }
}

