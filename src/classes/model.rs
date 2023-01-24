use crate::error::ReadResult;
use crate::gbx::{self, Class, ReadBody, ReadChunk, ReadChunkFn, ReadHeader};
use crate::reader::{self, Reader};
use crate::{Block, Item};
use std::borrow::BorrowMut;
use std::io::{Read, Seek};
use std::ops::{Deref, DerefMut};

/// Material of a model.
#[derive(Clone, Default)]
pub struct Material;

impl Material {
    fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
    {
        let mut material = Self::default();
        gbx::read_body(&mut material, r)?;
        Ok(material)
    }

    fn read_chunk_090fd000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
    {
        let version = r.u32()?;

        let is_game_material = if version >= 11 { r.bool_u8()? } else { false };
        r.optional_id()?;
        r.u32()?;
        r.u32()?;
        r.u8()?;
        r.u8()?;
        if version >= 11 && !is_game_material {
            r.id()?;
        } else {
            r.string()?;
        }
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

        Ok(())
    }

    fn read_chunk_090fd001<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
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

        Ok(())
    }

    fn read_chunk_090fd002<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;

        Ok(())
    }
}

impl<R, I, N> ReadBody<R, I, N> for Material
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x090FD000, ReadChunk::Read(Self::read_chunk_090fd000)),
            (0x090FD001, ReadChunk::Read(Self::read_chunk_090fd001)),
            (0x090FD002, ReadChunk::Read(Self::read_chunk_090fd002)),
        ]
    }
}

/// Model.
#[derive(Clone, Default)]
pub struct Model {
    /// Materials used in the model.
    pub materials: Vec<Material>,
}

#[derive(Clone, Default)]
pub(crate) struct Crystal(pub Model);

impl Crystal {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let mut crystal = Self::default();
        gbx::read_body(&mut crystal, r)?;
        Ok(crystal)
    }

    fn read_chunk_09051000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_09003003<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        r.u32()?;
        self.materials = r.list(|r| {
            r.u32()?;
            let material = r.node_owned(0x090FD000, Material::read)?;

            Ok(material)
        })?;

        Ok(())
    }

    fn read_chunk_09003005<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
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
                    read_mesh(r, self.materials.len() as u32)?;
                    r.list(|r| r.u32())?;
                    r.u32()?;
                    r.u32()?;
                }
                14 => {
                    read_mesh(r, self.materials.len() as u32)?;
                    r.list(|r| r.u32())?;
                }
                15 => {
                    r.u32()?;
                    r.u32()?;
                    r.vec3f32()?;
                    r.f32()?;
                    r.f32()?;
                    r.f32()?;
                }
                18 => {
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.node(0x090F9000, |r| {
                        r.chunk_id(0x090F9000)?;
                        r.u32()?;
                        r.u32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.f32()?;
                        r.u32()?;

                        r.node_end()?;

                        Ok(())
                    })?;
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
                }
                _ => panic!("{}", layer_type),
            }

            Ok(())
        })?;

        Ok(())
    }

    fn read_chunk_09003006<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
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

        Ok(())
    }

    fn read_chunk_09003007<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.list(|r| r.f32())?;
        r.list(|r| r.u32())?;

        Ok(())
    }
}

impl Deref for Crystal {
    type Target = Model;

    fn deref(&self) -> &Model {
        &self.0
    }
}

impl DerefMut for Crystal {
    fn deref_mut(&mut self) -> &mut Model {
        &mut self.0
    }
}

impl<R, I, N> ReadBody<R, I, N> for Crystal
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
    N: BorrowMut<reader::NodeState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x09051000, ReadChunk::Read(Self::read_chunk_09051000)),
            (0x09003003, ReadChunk::Read(Self::read_chunk_09003003)),
            (0x09003004, ReadChunk::Skip),
            (0x09003005, ReadChunk::Read(Self::read_chunk_09003005)),
            (0x09003006, ReadChunk::Read(Self::read_chunk_09003006)),
            (0x09003007, ReadChunk::Read(Self::read_chunk_09003007)),
        ]
    }
}

#[derive(Clone)]
pub(crate) enum ItemModel {
    Block(Block),
    Item(Item),
}

impl ItemModel {
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
        if let Some(item_model) = r.any_optional_node_owned(|r, class_id| {
            let item_model = match class_id {
                0x2E025000 => ItemModel::Block(Block::read(r)?),
                0x2E026000 => ItemModel::Item(Item::read(r)?),
                _ => panic!(),
            };

            Ok(item_model)
        })? {
            *self = item_model;
        }
        if let Some(item_model) = r.optional_node_owned(0x2E027000, |r| {
            r.chunk_id(0x2E027000)?;
            r.u32()?;
            let model = r.node_owned(0x09159000, |r| {
                r.u32()?;
                let model = r.node_owned(0x090BB000, |r| {
                    r.chunk_id(0x090BB000)?;
                    let version = r.u32()?;
                    r.u32()?;
                    r.list(|r| {
                        r.u32()?;
                        r.u32()?;
                        r.u32()?;
                        r.u32()?;

                        Ok(())
                    })?;
                    r.u32()?;
                    r.list(|r| {
                        r.node(0x0901E000, |r| {
                            r.chunk_id(0x09006001)?;
                            r.u32()?;

                            r.chunk_id(0x09006005)?;
                            r.u32()?;

                            r.chunk_id(0x09006009)?;
                            r.u32()?;

                            r.chunk_id(0x0900600B)?;
                            r.u32()?;

                            r.chunk_id(0x0900600F)?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.u32()?;
                            r.node(0x09056000, |r| {
                                r.chunk_id(0x09056000)?;
                                r.u32()?;
                                let num_vertices = r.u32()?;
                                r.u32()?;
                                r.u32()?;
                                let attributes = r.list(|r| {
                                    r.u8()?; // byte offset * 4
                                    r.u8()?;
                                    r.u8()?;
                                    r.u8()?;
                                    r.u8()?;
                                    r.u8()?;
                                    let _byte_offset = r.u8()?;
                                    r.u8()?;
                                    let kind = r.u8()?;
                                    r.u8()?;
                                    r.u8()?;
                                    r.u8()?;

                                    Ok(kind)
                                })?;
                                for kind in attributes {
                                    match kind {
                                        1 => {
                                            r.repeat(num_vertices as usize, |r| {
                                                r.f32()?;
                                                r.f32()?;

                                                Ok(())
                                            })?;
                                        }
                                        5 => {
                                            r.repeat(num_vertices as usize, |r| {
                                                r.f32()?;
                                                r.f32()?;
                                                r.f32()?;

                                                Ok(())
                                            })?;
                                        }
                                        10 => {
                                            r.repeat(num_vertices as usize, |r| r.u32())?;
                                        }
                                        11 => {
                                            r.repeat(num_vertices as usize, |r| {
                                                r.f32()?;
                                                r.f32()?;

                                                Ok(())
                                            })?;
                                        }
                                        18 => {
                                            r.repeat(num_vertices as usize, |r| r.f32())?;
                                        }
                                        20 => {
                                            r.repeat(num_vertices as usize, |r| r.f32())?;
                                        }
                                        _ => panic!(),
                                    }
                                }

                                r.node_end()?;

                                Ok(())
                            })?;
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

                            r.chunk_id(0x09006010)?;
                            r.u32()?;
                            r.u32()?;

                            r.chunk_id(0x0902C002)?;
                            r.u32()?;

                            r.chunk_id(0x0902C004)?;
                            r.u32()?;
                            r.u32()?;

                            r.chunk_id(0x0906A001)?;
                            r.u32()?;
                            {
                                r.chunk_id(0x09057001)?;
                                r.u32()?;
                                let mut current_index = 0;
                                let _indices = r.list(|r| {
                                    let offset = r.i16()?;

                                    if offset.is_positive() {
                                        current_index += offset as u16;
                                    } else {
                                        current_index -= (-offset) as u16;
                                    }

                                    Ok(current_index)
                                })?;

                                r.node_end()?;
                            }

                            r.node_end()?;

                            Ok(())
                        })?;

                        Ok(())
                    })?;
                    r.u32()?;
                    let num_materials = r.u32()?;
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
                    r.string()?; // "Stadium\Media\Material\"
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.string()?; // "*.Item.xml"
                    if version >= 30 {
                        r.u32()?;
                    }
                    let materials = r.repeat(num_materials as usize, |r| {
                        r.u32()?;
                        r.node_owned(0x090FD000, Material::read)
                    })?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;
                    r.u32()?;

                    r.skip_chunk(0x090BB002)?;

                    r.node_end()?;

                    Ok(Model { materials })
                })?;
                r.u8()?;
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

                r.node_end()?;

                Ok(model)
            })?;
            r.u32()?;

            Ok(ItemModel::Item(Item { model }))
        })? {
            *self = item_model;
        }
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

impl Default for ItemModel {
    fn default() -> Self {
        Self::Block(Block::default())
    }
}

impl Class for ItemModel {
    const CLASS_ID: u32 = 0x2E002000;
}

impl<R, I, N> ReadHeader<R, I, N> for ItemModel
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

impl<R, I, N> ReadBody<R, I, N> for ItemModel
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
