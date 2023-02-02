use crate::classes::model::Crystal;
use crate::error::{ReadError, ReadResult};
use crate::gbx::{ReadBody, ReadChunk};
use crate::model::{ItemModel, Model};
use crate::reader::Reader;
use crate::{gbx, reader};
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

/// Type corresponding to the file extension `Item.Gbx`.
#[derive(Clone, Default)]
pub struct Item {
    /// Model of the item.
    pub model: Model,
}

impl Item {
    /// Read a item from the given `reader`.
    ///
    /// For performance reasons, it is recommended that the `reader` is buffered.
    pub fn read_from<R>(reader: R) -> ReadResult<Self>
    where
        R: Read,
    {
        match gbx::read(reader)? {
            ItemModel::Item(item) => Ok(item),
            ItemModel::Block(_) => Err(ReadError(String::from("expected item, got block"))),
        }
    }

    /// Read a item from a file at the given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> gbx::error::ReadResult<()> {
    /// let item = gbx::Item::read_from_file("MyItem.Item.Gbx")?;
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
        let mut item = Self::default();
        gbx::read_body(&mut item, r)?;
        Ok(item)
    }

    fn read_chunk_2e026000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
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

impl<R, I, N> ReadBody<R, I, N> for Item
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
    N: BorrowMut<reader::NodeState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x2E026000, ReadChunk::Read(Self::read_chunk_2e026000)),
            (0x2E026001, ReadChunk::Skip),
        ]
    }
}
