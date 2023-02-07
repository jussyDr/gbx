use crate::model::{Crystal, ItemModel, Model};
use crate::read::{self, ReadBodyChunk, Reader, ReaderBuilder};
use crate::Id;
use std::borrow::BorrowMut;
use std::io::{Read, Seek};

/// Type corresponding to the file extension `Block.Gbx`.
#[derive(Clone, Default, Debug)]
pub struct Block {
    /// ID of the block info archetype.
    pub archetype: Id,
    /// Variant models of the block.
    pub variants: Vec<Model>,
}

impl Block {
    /// Get a block reader.
    pub fn reader() -> ReaderBuilder<Self> {
        ItemModel::<Self>::reader()
    }

    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> read::Result<Self>
    where
        R: Read + Seek,
        I: BorrowMut<read::IdState>,
        N: BorrowMut<read::NodeState>,
    {
        let mut block = Self::default();

        read::read_body(
            &mut block,
            r,
            vec![
                (0x2E025000, ReadBodyChunk::Read(Self::read_chunk_2e025000)),
                (0x2E025001, ReadBodyChunk::Skip),
                (0x2E025002, ReadBodyChunk::Skip),
                (0x2E025003, ReadBodyChunk::Skip),
            ],
        )?;

        Ok(block)
    }

    fn read_chunk_2e025000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> read::Result<()>
    where
        R: Read + Seek,
        I: BorrowMut<read::IdState>,
        N: BorrowMut<read::NodeState>,
    {
        r.u32()?;
        self.archetype = r.id()?;
        r.u32()?;
        self.variants = r.list(|r| {
            r.u32()?;
            let variant = r.node_owned(0x09003000, Crystal::read)?.0;

            Ok(variant)
        })?;

        Ok(())
    }
}
