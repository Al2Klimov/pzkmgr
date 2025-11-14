use std::fmt;

pub(crate) struct NullIntFmt(Option<i64>, &'static str);

impl NullIntFmt {
    pub(crate) fn new(x: Option<i64>, fallback: &'static str) -> Self {
        Self(x, fallback)
    }
}

impl fmt::Display for NullIntFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => write!(f, "{}", self.1),
            Some(x) => write!(f, "{}", x),
        }
    }
}
