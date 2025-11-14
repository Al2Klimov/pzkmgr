use std::fmt;

pub(crate) struct HexFmt<'a>(&'a [u8]);

impl<'a> HexFmt<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for HexFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }

        Ok(())
    }
}
