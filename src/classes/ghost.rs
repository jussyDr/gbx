use crate::error::ReadResult;
use crate::gbx::{self, ReadBody, ReadChunk};
use crate::reader::{self, Reader};
use std::borrow::BorrowMut;
use std::io::{Read, Seek};

/// Entity record.
#[derive(Default)]
pub struct EntityRecord;

impl EntityRecord {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        let mut entity_record = Self::default();

        r.chunk_id(0x0911F000)?;
        entity_record.read_chunk_0911f000(r)?;

        r.node_end()?;

        Ok(entity_record)
    }

    fn read_chunk_0911f000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
    {
        r.u32()?;
        let _size = r.u32()?;
        let compressed_size = r.u32()?;
        r.skip(compressed_size as u64)?;

        Ok(())
    }
}

/// Type corresponding to the file extension `Ghost.Gbx`.
#[derive(Default)]
pub struct Ghost;

impl Ghost {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let mut ghost = Self::default();
        gbx::read_body(&mut ghost, r)?;
        Ok(ghost)
    }

    fn read_chunk_0303f006<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        r.u16()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u16()?;

        Ok(())
    }

    fn read_chunk_03092000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let version = r.u32()?;
        r.id()?;
        r.u32()?;
        r.id()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.list(|r| {
            r.optional_file_ref()?;

            Ok(())
        })?;
        r.u32()?;
        r.string()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.node(0x0911F000, EntityRecord::read)?;
        r.list(|r| {
            r.u32()?;

            Ok(())
        })?;
        r.u32()?;
        r.u16()?;
        r.u8()?;
        r.string()?;
        if version >= 8 {
            r.string()?;
        }

        Ok(())
    }

    fn read_chunk_0309200c<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_0309200e<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_0309200f<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.string()?;

        Ok(())
    }

    fn read_chunk_03092010<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        r.id()?;

        Ok(())
    }

    fn read_chunk_0309201c<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
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

impl<R, I, N> ReadBody<R, I, N> for Ghost
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
    N: BorrowMut<reader::NodeState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x0303F006, ReadChunk::Read(Self::read_chunk_0303f006)),
            (0x0303F007, ReadChunk::Skip),
            (
                0x03092000,
                ReadChunk::ReadSkippable(Self::read_chunk_03092000),
            ),
            (0x03092005, ReadChunk::Skip),
            (0x03092008, ReadChunk::Skip),
            (0x0309200A, ReadChunk::Skip),
            (0x0309200B, ReadChunk::Skip),
            (0x0309200C, ReadChunk::Read(Self::read_chunk_0309200c)),
            (0x0309200E, ReadChunk::Read(Self::read_chunk_0309200e)),
            (0x0309200F, ReadChunk::Read(Self::read_chunk_0309200f)),
            (0x03092010, ReadChunk::Read(Self::read_chunk_03092010)),
            (0x03092013, ReadChunk::Skip),
            (0x03092014, ReadChunk::Skip),
            (0x0309201A, ReadChunk::Skip),
            (0x0309201B, ReadChunk::Skip),
            (0x0309201C, ReadChunk::Read(Self::read_chunk_0309201c)),
            (0x0309201D, ReadChunk::Skip),
            (0x03092022, ReadChunk::Skip),
            (0x03092023, ReadChunk::Skip),
            (0x03092024, ReadChunk::Skip),
            (0x03092025, ReadChunk::Skip),
            (0x03092026, ReadChunk::Skip),
            (0x03092027, ReadChunk::Skip),
            (0x03092028, ReadChunk::Skip),
            (0x03092029, ReadChunk::Skip),
            (0x0309202A, ReadChunk::Skip),
            (0x0309202B, ReadChunk::Skip),
            (0x0309202C, ReadChunk::Skip),
            (0x0309202D, ReadChunk::Skip),
        ]
    }
}
