use crate::error::{ReadError, ReadResult};
use crate::gbx::{self, ReadBody, ReadChunk};
use crate::model::{Crystal, ItemModel, Model};
use crate::reader::{self, Reader};
use crate::RcStr;
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

/// Type corresponding to the file extension `Block.Gbx`.
#[derive(Clone, Default)]
pub struct Block {
    /// ID of the block info archetype.
    pub archetype: RcStr,
    /// Variant models of the block.
    pub variants: Vec<Model>,
}

impl Block {
    /// Read a block from the given `reader`.
    ///
    /// For performance reasons, it is recommended that the `reader` is buffered.
    pub fn read_from<R>(reader: R) -> ReadResult<Self>
    where
        R: Read,
    {
        match gbx::read(reader)? {
            ItemModel::Block(block) => Ok(block),
            ItemModel::Item(_) => Err(ReadError(String::from("expected block, got item"))),
        }
    }

    /// Read a block from a file at the given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> gbx::error::ReadResult<()> {
    /// let block = gbx::Block::read_from_file("MyBlock.Block.Gbx")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_from_file<P>(path: P) -> ReadResult<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        Self::read_from(reader)
    }

    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let mut block = Self::default();
        gbx::read_body(&mut block, r)?;
        Ok(block)
    }

    fn read_chunk_2e025000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
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

impl<R, I, N> ReadBody<R, I, N> for Block
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
    N: BorrowMut<reader::NodeState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x2E025000, ReadChunk::Read(Self::read_chunk_2e025000)),
            (0x2E025001, ReadChunk::Skip),
            (0x2E025002, ReadChunk::Skip),
            (0x2E025003, ReadChunk::Skip),
        ]
    }
}
