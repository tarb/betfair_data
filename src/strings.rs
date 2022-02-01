use staticvec::StaticString;

pub trait StringSetExtNeq {
    fn set_if_ne(&mut self, s: &str) -> bool;
}

impl StringSetExtNeq for String {
    fn set_if_ne(&mut self, s: &str) -> bool {
        if s != self {
            self.clear();
            self.push_str(s);
            true
        } else {
            false
        }
    }
}

impl StringSetExtNeq for Option<String> {
    fn set_if_ne(&mut self, s: &str) -> bool {
        match self {
            Some(str) => str.set_if_ne(s),
            None => {
                *self = Some(String::from(s));
                true
            }
        }
    }
}

impl<const N: usize> StringSetExtNeq for StaticString<N> {
    fn set_if_ne(&mut self, s: &str) -> bool {
        if s != self.as_str() {
            self.clear();
            self.push_str(s);
            true
        } else {
            false
        }
    }
}

impl<const N: usize> StringSetExtNeq for Option<StaticString<N>> {
    fn set_if_ne(&mut self, s: &str) -> bool {
        match self {
            Some(str) => str.set_if_ne(s),
            None => {
                *self = Some(StaticString::from(s));
                true
            }
        }
    }
}
