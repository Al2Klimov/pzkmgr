use std::fmt;

pub(crate) struct NullIntFmt(Option<i64>);

impl NullIntFmt {
    pub(crate) fn new(x: Option<i64>) -> Self {
        Self(x)
    }
}

impl fmt::Display for NullIntFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            None => write!(f, "NULL"),
            Some(x) => write!(f, "{}", x),
        }
    }
}
