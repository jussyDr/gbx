use crate::error::ReadResult;
use crate::gbx::{self, Class, ReadBody, ReadChunk, ReadChunkFn, ReadHeader};
use crate::reader::{self, Reader};
use crate::RcStr;
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

/// Type corresponding to the file extension `Block.Gbx`.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> gbx::error::ReadResult<()> {
/// let block = gbx::Block::read_from_file("MyBlock.Block.Gbx")?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct Block {
    /// Id of the block info archetype.
    pub archetype: RcStr,
}

impl Block {
    pub fn read_from<R>(reader: R) -> ReadResult<Self>
    where
        R: Read,
    {
        gbx::read(reader)
    }

    pub fn read_from_file<P>(path: P) -> ReadResult<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        Self::read_from(reader)
    }

    fn read_chunk_2e001003<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        r.optional_id()?;
        r.u32()?;
        r.id()?;
        r.u32()?;
        let _name = r.string()?;
        r.u32()?;
        r.u32()?;
        r.u16()?;
        r.string()?;
        r.u8()?;

        Ok(())
    }

    fn read_chunk_2e001004<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let icon_width = r.u16()?;
        let icon_height = r.u16()?;
        r.repeat(icon_width as usize * icon_height as usize, |r| r.u32())?;

        Ok(())
    }

    fn read_chunk_2e001006<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u64()?;

        Ok(())
    }

    fn read_chunk_2e001009<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        let _name = r.string()?;
        r.u32()?;
        r.optional_id()?;

        Ok(())
    }

    fn read_chunk_2e00100b<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        r.u32()?;
        r.u32()?;
        r.id()?;

        Ok(())
    }

    fn read_chunk_2e00100c<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.string()?;

        Ok(())
    }

    fn read_chunk_2e00100d<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.string()?;

        Ok(())
    }

    fn read_chunk_2e00100e<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e001010<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e001011<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u8()?;

        Ok(())
    }

    fn read_chunk_2e002000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let _item_type = r.u32()?;

        Ok(())
    }

    fn read_chunk_2e002001<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e002008<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.list(|r| r.u32())?;

        Ok(())
    }

    fn read_chunk_2e002009<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e00200c<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e002012<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.f32()?;
        r.f32()?;

        Ok(())
    }

    fn read_chunk_2e002015<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let _item_type = r.u32()?;

        Ok(())
    }

    fn read_chunk_2e002019<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let version = r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.node(0x2E025000, |r| {
            r.chunk_id(0x2E025000)?;
            r.u32()?;
            self.archetype = r.id()?;
            r.u32()?;
            r.list(|r| {
                r.u32()?;
                r.node(0x09003000, |r| {
                    r.chunk_id(0x09051000)?;
                    r.u32()?;

                    r.chunk_id(0x09003003)?;
                    r.u32()?;
                    let materials = r.list(|r| {
                        r.u32()?;
                        r.node(0x090FD000, |r| {
                            r.chunk_id(0x090FD000)?;
                            let version = r.u32()?;
                            if version >= 11 {
                                r.u8()?;
                            }
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u8()?;
                            r.u8()?;
                            let _name = r.string()?;
                            r.list(|r| {
                                r.id()?;
                                r.id()?;
                                r.u32()?;

                                Ok(())
                            })?;
                            r.list(|r| r.u32())?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;

                            r.chunk_id(0x090FD001)?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;

                            r.chunk_id(0x090FD002)?;
                            r.u32()?;
                            r.u32()?;

                            Ok(())
                        })?;

                        r.node_end()?;

                        Ok(())
                    })?;

                    r.skip_chunk(0x09003004)?;

                    r.chunk_id(0x09003005)?;
                    r.u32()?;
                    let _layers = r.list(|r| {
                        let layer_type = r.u32()?;
                        r.u32()?;
                        r.u32()?;
                        r.id()?;
                        let _name = r.string()?;
                        let _is_enabled = r.bool()?;
                        r.u32()?;

                        match layer_type {
                            0 => {
                                read_mesh(r, materials.len() as u32)?;
                                r.list(|r| r.u32())?;
                                r.u32()?;
                                r.u32()?;
                            }
                            14 => {
                                read_mesh(r, materials.len() as u32)?;
                                r.list(|r| r.u32())?;
                            }
                            _ => panic!("{}", layer_type),
                        }

                        Ok(())
                    })?;

                    r.chunk_id(0x09003006)?;
                    let version = r.u32()?;
                    if version >= 1 {
                        r.list(|r| {
                            r.i16()?;
                            r.i16()?;

                            Ok(())
                        })?;
                        if version >= 2 {
                            let num = r.u32()?;
                            r.repeat(num as usize, |r| {
                                r.packed_index(num)?;

                                Ok(())
                            })?;
                        }
                    } else {
                        r.list(|r| {
                            r.f32()?;
                            r.f32()?;

                            Ok(())
                        })?;
                    }

                    r.chunk_id(0x09003007)?;
                    r.u32()?;
                    r.list(|r| r.f32())?;
                    r.list(|r| r.u32())?;

                    Ok(())
                })?;

                r.node_end()?;

                Ok(())
            })?;

            r.skip_optional_chunk(0x2E025001)?;
            r.skip_chunk(0x2E025002)?;
            r.skip_chunk(0x2E025003)?;

            r.node_end()?;

            Ok(())
        })?;
        r.u32()?;
        if version >= 15 {
            r.u32()?;
        }

        Ok(())
    }

    fn read_chunk_2e00201a<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e00201c<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        N: BorrowMut<reader::NodeState>,
    {
        r.u32()?;
        r.node(0x2E020000, |r| {
            r.skip_chunk(0x2E020000)?;
            r.skip_chunk(0x2E020001)?;
            r.skip_chunk(0x2E020003)?;
            r.skip_optional_chunk(0x2E020004)?;

            Ok(())
        })?;

        Ok(())
    }

    fn read_chunk_2e00201e<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let version = r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        if version >= 7 {
            r.u32()?;
        }

        Ok(())
    }

    fn read_chunk_2e00201f<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let version = r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        if version >= 11 {
            r.u8()?;
        }

        Ok(())
    }

    fn read_chunk_2e002020<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        let _icon_path = r.string()?;
        r.u8()?;

        Ok(())
    }

    fn read_chunk_2e002021<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_2e002023<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u8()?;
        r.u32()?;

        Ok(())
    }
}

impl Class for Block {
    const CLASS_ID: u32 = 0x2E002000;
}

impl<R, I, N> ReadHeader<R, I, N> for Block
where
    R: Read,
    I: BorrowMut<reader::IdState>,
{
    fn header_chunks<'a>() -> &'a [(u32, ReadChunkFn<Self, R, I, N>)] {
        &[
            (0x2e001003, Self::read_chunk_2e001003),
            (0x2e001004, Self::read_chunk_2e001004),
            (0x2e001006, Self::read_chunk_2e001006),
            (0x2e002000, Self::read_chunk_2e002000),
            (0x2e002001, Self::read_chunk_2e002001),
        ]
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
            (0x2E001009, ReadChunk::Read(Self::read_chunk_2e001009)),
            (0x2E00100B, ReadChunk::Read(Self::read_chunk_2e00100b)),
            (0x2E00100C, ReadChunk::Read(Self::read_chunk_2e00100c)),
            (0x2E00100D, ReadChunk::Read(Self::read_chunk_2e00100d)),
            (0x2E00100E, ReadChunk::Read(Self::read_chunk_2e00100e)),
            (0x2E001010, ReadChunk::Read(Self::read_chunk_2e001010)),
            (0x2E001011, ReadChunk::Read(Self::read_chunk_2e001011)),
            (0x2E002008, ReadChunk::Read(Self::read_chunk_2e002008)),
            (0x2E002009, ReadChunk::Read(Self::read_chunk_2e002009)),
            (0x2E00200C, ReadChunk::Read(Self::read_chunk_2e00200c)),
            (0x2E002012, ReadChunk::Read(Self::read_chunk_2e002012)),
            (0x2E002015, ReadChunk::Read(Self::read_chunk_2e002015)),
            (0x2E002019, ReadChunk::Read(Self::read_chunk_2e002019)),
            (0x2E00201A, ReadChunk::Read(Self::read_chunk_2e00201a)),
            (0x2E00201C, ReadChunk::Read(Self::read_chunk_2e00201c)),
            (0x2E00201E, ReadChunk::Read(Self::read_chunk_2e00201e)),
            (0x2E00201F, ReadChunk::Read(Self::read_chunk_2e00201f)),
            (0x2E002020, ReadChunk::Read(Self::read_chunk_2e002020)),
            (0x2E002021, ReadChunk::Read(Self::read_chunk_2e002021)),
            (0x2E002023, ReadChunk::Read(Self::read_chunk_2e002023)),
            (0x2E002024, ReadChunk::Skip),
            (0x2E002025, ReadChunk::Skip),
            (0x2E002026, ReadChunk::Skip),
            (0x2E002027, ReadChunk::Skip),
        ]
    }
}

fn read_mesh<R, I, N>(r: &mut Reader<R, I, N>, num_materials: u32) -> ReadResult<()>
where
    R: Read,
{
    let version = r.u32()?;
    r.u32()?;
    r.u32()?;
    r.u32()?;
    r.f32()?;
    r.u32()?;
    r.f32()?;
    r.u32()?;
    r.f32()?;
    r.u32()?;
    let groups = r.list(|r| {
        r.u32()?;
        if version >= 36 {
            r.u8()?;
        } else {
            r.u32()?;
        }
        r.u32()?;
        let _name = r.string()?;
        r.u32()?;
        r.list(|r| r.u32())?;

        Ok(())
    })?;
    if version >= 34 {
        r.u8()?;
    } else {
        r.u32()?;
    }
    if version >= 33 {
        r.u32()?;
        r.u32()?;
    }
    let positions = r.list(|r| {
        r.f32()?;
        r.f32()?;
        r.f32()?;

        Ok(())
    })?;
    let num_edges = r.u32()?;
    if version >= 35 {
        r.u32()?;
    } else {
        let _edges = r.repeat(num_edges as usize, |r| {
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;
    }
    let num_faces = r.u32()?;
    if version >= 37 {
        let _texcoords = r.list(|r| {
            r.f32()?;
            r.f32()?;

            Ok(())
        })?;
        let num_face_indices = r.u32()?;
        r.repeat(num_face_indices as usize, |r| {
            r.packed_index(num_face_indices)?;

            Ok(())
        })?;
    }
    let _faces = r.repeat(num_faces as usize, |r| {
        let num_vertices = if version >= 35 {
            r.u8()? as u32 + 3
        } else {
            r.u32()?
        };
        if version >= 34 {
            r.repeat(num_vertices as usize, |r| {
                r.packed_index(positions.len() as u32)?;

                Ok(())
            })?;
        } else {
            let _indices = r.repeat(num_vertices as usize, |r| r.u32())?;
        }
        if version < 37 {
            let _texcoords = r.repeat(num_vertices as usize, |r| {
                r.f32()?;
                r.f32()?;

                Ok(())
            })?;
        }
        if version >= 33 {
            r.packed_index(num_materials)?;
            r.packed_index(groups.len() as u32)?;
        } else {
            let _material_index = r.u32()?;
            let _group_index = r.u32()?;
        }

        Ok(())
    })?;
    r.u32()?;
    if version < 36 {
        let num_faces = r.u32()?;
        let num_edges = r.u32()?;
        let num_vertices = r.u32()?;
        r.repeat(num_faces as usize, |r| r.u32())?;
        r.repeat(num_edges as usize, |r| r.u32())?;
        r.repeat(num_vertices as usize, |r| r.u32())?;
        r.u32()?;
    }

    Ok(())
}
