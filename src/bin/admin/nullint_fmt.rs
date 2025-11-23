use std::fmt;

#[derive(Clone)]
pub(crate) struct NullIntFmt {
    value: Option<i64>,
    width: Option<usize>,
    fallback: &'static str,
}

impl NullIntFmt {
    pub(crate) fn new(value: Option<i64>, width: Option<usize>, fallback: &'static str) -> Self {
        Self {
            value,
            width,
            fallback,
        }
    }
}

impl fmt::Display for NullIntFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            None => write!(f, "{}", self.fallback),
            Some(v) => match self.width {
                None => write!(f, "{}", v),
                Some(w) => write!(f, "{:0>w$}", v),
            },
        }
    }
}
