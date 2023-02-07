use crate::classes::model::Crystal;
use crate::model::{ItemModel, Model};
use crate::read::{self, ReadBodyChunk, Reader, ReaderBuilder};
use std::borrow::BorrowMut;
use std::io::{Read, Seek};

/// Type corresponding to the file extension `Item.Gbx`.
#[derive(Clone, Default, Debug)]
pub struct Item {
    /// Model of the item.
    pub model: Model,
}

impl Item {
    /// Get a item reader.
    pub fn reader() -> ReaderBuilder<Self> {
        ItemModel::<Self>::reader()
    }

    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> read::Result<Self>
    where
        R: Read + Seek,
        I: BorrowMut<read::IdState>,
        N: BorrowMut<read::NodeState>,
    {
        let mut item = Self::default();

        read::read_body(
            &mut item,
            r,
            vec![
                (0x2E026000, ReadBodyChunk::Read(Self::read_chunk_2e026000)),
                (0x2E026001, ReadBodyChunk::Skip),
            ],
        )?;

        Ok(item)
    }

    fn read_chunk_2e026000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> read::Result<()>
    where
        R: Read + Seek,
        I: BorrowMut<read::IdState>,
        N: BorrowMut<read::NodeState>,
    {
        r.u32()?;
        r.u32()?;
        self.model = r.node_owned(0x09003000, Crystal::read)?.0;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        Ok(())
    }
}
