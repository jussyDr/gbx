use crate::error::WriteResult;
use std::io::Write;

pub struct Writer<W> {
    inner: W,
}

impl<W> Writer<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

macro_rules! impl_write_num {
    ($($type:ident),+) => {
        $(
            pub fn $type(&mut self, val: $type) -> WriteResult {
                self.bytes(&val.to_le_bytes())
            }
        )+
    };
}

impl<W> Writer<W>
where
    W: Write,
{
    pub fn bytes(&mut self, bytes: &[u8]) -> WriteResult {
        self.inner.write_all(bytes).map_err(Into::into)
    }

    impl_write_num!(u8, u16, u32);
}
