/// Media tracker types.
pub mod media;

use crate::error::{ReadResult, WriteResult};
use crate::gbx::{ReadBody, ReadChunk};
use crate::header::{Compression, Header};
use crate::reader::{self, Reader};
use crate::ref_table::RefTable;
use crate::types::{RcStr, Vec3};
use crate::writer::{self, Writer};
use crate::{gbx, FileRef, Ghost};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, Write};
use std::ops::Sub;
use std::path::Path;

/// Cardinal direction of a block.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}

impl Sub for Direction {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::try_from(((u8::from(self) + 4) - u8::from(rhs)) % 4).unwrap()
    }
}

/// Color of a block/item.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Color {
    #[default]
    Default,
    White,
    Green,
    Blue,
    Red,
    Black,
}

/// Lightmap quality of a block/item.
#[derive(Clone, Copy, Default, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LightmapQuality {
    #[default]
    Normal,
    High,
    VeryHigh,
    Highest,
    Low,
    VeryLow,
    Lowest,
}

/// Animation phase offset of a moving item.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PhaseOffset {
    #[default]
    None,
    One8th,
    Two8th,
    Three8th,
    Four8th,
    Five8th,
    Six8th,
    Seven8th,
}

/// Skin of a block/item.
#[derive(Clone, Default)]
pub struct Skin {
    /// The skin.
    pub skin: Option<FileRef>,
    /// Additional effect overlayed on top of the skin.
    pub effect: Option<FileRef>,
}

impl Skin {
    fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let mut skin = Self::default();
        gbx::read_body(&mut skin, r)?;
        Ok(skin)
    }

    fn read_chunk_03059002<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?; // 2
        r.u16()?;
        self.skin = r.optional_file_ref()?;
        r.optional_file_ref()?;

        Ok(())
    }

    fn read_chunk_03059003<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?; // 0
        self.effect = r.optional_file_ref()?;

        Ok(())
    }

    fn write<W, I, N>(&self, w: &mut Writer<W, I, N>) -> WriteResult
    where
        W: Write,
    {
        w.chunk_id(0x03059002)?;
        w.u32(2)?;
        w.u16(13345)?;
        if let Some(skin) = &self.skin {
            w.file_ref(skin)?;
        } else {
            w.null_file_ref()?;
        }
        w.null_file_ref()?;

        w.chunk_id(0x03059003)?;
        w.u32(0)?;
        if let Some(effect) = &self.effect {
            w.file_ref(effect)?;
        } else {
            w.null_file_ref()?;
        }

        Ok(())
    }
}

impl<R, I, N> ReadBody<R, I, N> for Skin
where
    R: Read,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x03059002, ReadChunk::Read(Self::read_chunk_03059002)),
            (0x03059003, ReadChunk::Read(Self::read_chunk_03059003)),
        ]
    }
}

/// Order of a start, finish or multilap block/item in royal.
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum RoyalOrder {
    /// First.
    White = 1,
    /// Second.
    Green,
    /// Third.
    Blue,
    /// Fourth.
    Red,
    /// Fifth.
    Black,
}

/// Waypoint property of a block/item.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum WaypointProperty {
    /// Checkpoint.
    Checkpoint,
    /// Linked checkpoint.
    LinkedCheckpoint {
        /// Group number.
        group: u32,
    },
    /// Start.
    Start {
        /// Order for royal.
        order: Option<RoyalOrder>,
    },
    /// Finish.
    Finish {
        /// Order for royal.
        order: Option<RoyalOrder>,
    },
    /// Multilap.
    StartFinish {
        /// Order for royal.
        order: Option<RoyalOrder>,
    },
}

impl WaypointProperty {
    fn read<R, I, S>(r: &mut Reader<R, I, S>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x2E009000)?;
        r.u32()?; // 2
        let tag = r.string()?;
        let variant = match tag.as_str() {
            "Checkpoint" => {
                r.skip(4)?;
                Self::Checkpoint
            }
            "LinkedCheckpoint" => Self::LinkedCheckpoint { group: r.u32()? },
            "Spawn" => Self::Start {
                order: RoyalOrder::try_from(r.u32()?).ok(),
            },
            "Goal" => Self::Finish {
                order: RoyalOrder::try_from(r.u32()?).ok(),
            },
            "StartFinish" => Self::StartFinish {
                order: RoyalOrder::try_from(r.u32()?).ok(),
            },
            _ => panic!(),
        };

        r.skip_chunk(0x2E009001)?;

        r.node_end()?;

        Ok(variant)
    }

    fn write<W, I, S>(&self, w: &mut Writer<W, I, S>) -> WriteResult
    where
        W: Write,
        I: Clone,
        S: Clone,
    {
        w.chunk_id(0x2E009000)?;
        w.u32(2)?;
        match *self {
            Self::Checkpoint => {
                w.string("Checkpoint")?;
                w.u32(0)?;
            }
            Self::LinkedCheckpoint { group } => {
                w.string("LinkedCheckpoint")?;
                w.u32(group)?;
            }
            Self::Start { order } => {
                w.string("Start")?;
                w.u32(order.map(|order| order.into()).unwrap_or(0))?;
            }
            Self::Finish { order } => {
                w.string("Finish")?;
                w.u32(order.map(|order| order.into()).unwrap_or(0))?;
            }
            Self::StartFinish { order } => {
                w.string("StartFinish")?;
                w.u32(order.map(|order| order.into()).unwrap_or(0))?;
            }
        }

        w.skippable_chunk(0x2E009001, |w| {
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        Ok(())
    }
}

/// A block inside of a `Map`.
#[derive(Default)]
pub struct Block {
    /// Id of the block's model.
    pub id: RcStr,
    /// Direction of the block.
    pub dir: Direction,
    /// Coordinate of the block.
    pub coord: Vec3<u8>,
    /// `true` if the block is a ground block variant.
    pub is_ground: bool,
    /// Skin of the block, e.g. for signs.
    pub skin: Option<Skin>,
    /// Waypoint property.
    pub waypoint_property: Option<WaypointProperty>,
    /// Variant index of the block.
    pub variant_index: u8,
    /// `true` if the block is a ghost block.
    pub is_ghost: bool,
    /// Color of the block.
    pub color: Color,
    /// Lightmap quality of the block.
    pub lightmap_quality: LightmapQuality,
}

/// A free block inside of a `Map`.
#[derive(Default)]
pub struct FreeBlock {
    /// Id of the block's model.
    pub id: RcStr,
    /// Skin of the block, e.g. for signs.
    pub skin: Option<Skin>,
    /// Waypoint property.
    pub waypoint_property: Option<WaypointProperty>,
    /// Position of the block.
    pub pos: Vec3<f32>,
    /// Yaw rotation of the block.
    pub yaw: f32,
    /// Pitch rotation of the block.
    pub pitch: f32,
    /// Roll rotation of the block.
    pub roll: f32,
    /// Color of the block.
    pub color: Color,
    /// Lightmap quality of the block.
    pub lightmap_quality: LightmapQuality,
}

enum BlockType {
    Normal(Block),
    Free(FreeBlock),
}

impl Default for BlockType {
    fn default() -> Self {
        Self::Normal(Block::default())
    }
}

/// An item inside of a `Map`.
#[derive(Default)]
pub struct Item {
    /// Id of the item's model.
    pub id: RcStr,
    /// Yaw rotation of the item.
    pub yaw: f32,
    /// Pitch rotation of the item.
    pub pitch: f32,
    /// Roll rotation of the item.
    pub roll: f32,
    /// Coord inside the map.
    pub coord: Vec3<u8>,
    /// Position inside the map.
    pub pos: Vec3<f32>,
    /// Waypoint property.
    pub waypoint_property: Option<WaypointProperty>,
    /// Pivot position of the item.
    pub pivot_pos: Vec3<f32>,
    /// Color of the item.
    pub color: Color,
    /// Phase offset of the item's animation.
    pub anim_offset: PhaseOffset,
    /// Lightmap quality of the item.
    pub lightmap_quality: LightmapQuality,
}

impl Item {
    fn read<R>(r: &mut Reader<R, reader::IdState>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x03101002)?;
        r.u32()?; // 8
        let id = r.id()?;
        r.u32()?; // 26
        let _author = r.optional_id()?; // "Nadeo"
        let yaw = r.f32()?;
        let pitch = r.f32()?;
        let roll = r.f32()?;
        let coord_x = r.u8()?;
        let coord_y = r.u8()?;
        let coord_z = r.u8()?;
        r.u32()?; // 0xFFFFFFFF
        let x = r.f32()?;
        let y = r.f32()?;
        let z = r.f32()?;
        let waypoint_property = r.optional_flat_node(0x2E009000, WaypointProperty::read)?;
        let flags = r.u16()?;
        let pivot_x = r.f32()?;
        let pivot_y = r.f32()?;
        let pivot_z = r.f32()?;
        let _scale = r.f32()?;
        if flags & 0x0004 != 0 {
            r.optional_file_ref()?;
        }
        r.u32()?; // 0
        r.u32()?; // 0
        r.u32()?; // 0
        r.f32()?; // -1.0
        r.f32()?; // -1.0
        r.f32()?; // -1.0

        r.skip_optional_chunk(0x03101004)?;
        r.skip_optional_chunk(0x03101005)?;

        r.node_end()?;

        Ok(Self {
            id,
            yaw,
            pitch,
            roll,
            coord: Vec3 {
                x: coord_x,
                y: coord_y,
                z: coord_z,
            },
            pos: Vec3 { x, y, z },
            waypoint_property,
            pivot_pos: Vec3 {
                x: pivot_x,
                y: pivot_y,
                z: pivot_z,
            },
            ..Default::default()
        })
    }

    fn write<W>(&self, w: &mut Writer<W, writer::IdState>) -> WriteResult
    where
        W: Write,
    {
        w.chunk_id(0x03101002)?;
        w.u32(8)?;
        w.id(&self.id)?;
        w.u32(26)?;
        w.null_id()?;
        w.f32(self.yaw)?;
        w.f32(self.pitch)?;
        w.f32(self.roll)?;
        w.u8(self.coord.x)?;
        w.u8(self.coord.y)?;
        w.u8(self.coord.z)?;
        w.u32(0xFFFFFFFF)?;
        w.f32(self.pos.x)?;
        w.f32(self.pos.y)?;
        w.f32(self.pos.z)?;
        if let Some(waypoint_property) = &self.waypoint_property {
            w.flat_node(0x2E009000, |w| waypoint_property.write(w))?;
        } else {
            w.null_node()?;
        }
        w.u16(0)?;
        w.f32(self.pivot_pos.x)?;
        w.f32(self.pivot_pos.y)?;
        w.f32(self.pivot_pos.z)?;
        w.f32(1.0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.f32(-1.0)?;
        w.f32(-1.0)?;
        w.f32(-1.0)?;

        Ok(())
    }
}

/// Type corresponding to the file extension `Map.Gbx`.
#[derive(Default)]
pub struct Map {
    /// Bronze medal time in milliseconds.
    pub bronze_time: u32,
    /// Silver medal time in milliseconds.
    pub silver_time: u32,
    /// Gold medal time in milliseconds.
    pub gold_time: u32,
    /// Author medal time in milliseconds.
    pub author_time: u32,
    /// Display cost of the map.
    pub cost: u32,
    /// Number of checkpoints needed to finish the map.
    pub num_cps: u32,
    /// Number of laps if multilap.
    pub num_laps: Option<u32>,
    /// Name of the map.
    pub name: String,
    /// Name of the map decoration.
    pub deco_name: RcStr,
    /// Optional thumbnail of the map as raw JPEG.
    pub thumbnail: Option<Vec<u8>>,
    /// Name of the map author.
    pub author_name: String,
    /// Zone of the map author.
    pub author_zone: String,

    /// `true` if the map has been validated.
    pub is_validated: bool,
    /// Optional texture mod.
    pub texture_mod: Option<FileRef>,
    /// Size of the map.
    pub size: Vec3<u32>,
    /// All blocks placed inside of the map.
    pub blocks: Vec<Block>,
    /// All free blocks placed inside of the map.
    pub free_blocks: Vec<FreeBlock>,
    /// Optional map music.
    pub music: Option<FileRef>,
    /// All items placed inside of the map.
    pub items: Vec<Item>,
    /// Optional MediaTracker clip for the map intro.
    pub intro_media: Option<media::Clip>,
    /// Optional MediaTracker clip for the podium.
    pub podium_media: Option<media::Clip>,
    /// Optional MediaTracker clips for in game.
    pub in_game_media: Option<media::ClipGroup>,
    /// Optional MediaTracker clips for end race.
    pub end_race_media: Option<media::ClipGroup>,
    /// Optional MediaTracker clip for the map ambiance.
    pub ambiance_media: Option<media::Clip>,
    /// Id's of the files embedded in the map.
    ///
    /// The length is equal to the number of files in the `embedded_files` ZIP archive.
    pub embedded_file_ids: Vec<RcStr>,
    /// All files embedded in the map as a raw ZIP archive.
    pub embedded_files: Vec<u8>,

    baked_blocks: Vec<BlockType>,
}

impl Map {
    /// Create a new map with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a map from the given `reader`.
    ///
    /// For performance reasons, it is recommended that the `reader` is buffered.
    pub fn read_from<R>(reader: R) -> ReadResult<Self>
    where
        R: Read,
    {
        let mut map = Map::default();

        let mut r = Reader::new(reader);
        let header = Header::read(&mut r, 0x03043000)?;

        if !header.user_data.is_empty() {
            let mut r = Reader::with_id_state(
                Cursor::new(header.user_data.as_slice()),
                reader::IdState::new(),
            );

            let _num_header_chunks = r.u32()?;

            r.chunk_id(0x03043002)?;
            let _chunk_size = r.u32()?;

            r.chunk_id(0x03043003)?;
            let _chunk_size = r.u32()?;

            r.chunk_id(0x03043004)?;
            let _chunk_size = r.u32()?;

            r.chunk_id(0x03043005)?;
            let _chunk_size = r.u32()?;

            r.chunk_id(0x03043007)?;
            let _chunk_size = r.u32()?;

            r.chunk_id(0x03043008)?;
            let _chunk_size = r.u32()?;

            r.u8()?;
            r.u32()?;
            map.bronze_time = r.u32()?;
            map.silver_time = r.u32()?;
            map.gold_time = r.u32()?;
            map.author_time = r.u32()?;
            map.cost = r.u32()?;
            let is_multilap = r.bool()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            map.num_cps = r.u32()?;
            if is_multilap {
                map.num_laps = Some(r.u32()?)
            } else {
                r.skip(4)?;
            }

            r.u8()?;
            r.id()?;
            r.u32()?;
            r.id()?;
            map.name = r.string()?;
            r.u8()?;
            r.u32()?;
            r.u32()?;
            map.deco_name = r.id()?;
            r.u32()?;
            let _deco_author = r.id()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            let _map_type = r.string()?;
            let _map_style = r.string()?;
            r.u64()?;
            r.u8()?;
            let _title_name = r.id()?;

            let _version = r.u32()?;

            let _xml = r.string()?;

            if r.bool()? {
                let thumbnail_size = r.u32()?;
                r.skip(15)?;
                map.thumbnail = Some(r.bytes(thumbnail_size as usize)?);
                r.skip(16)?;
                r.skip(10)?;
                let _map_comments = r.string()?;
                r.skip(11)?;
            }

            r.u32()?;
            r.u32()?;
            let _author_login = r.string()?;
            map.author_name = r.string()?;
            map.author_zone = r.string()?;
            r.u32()?;
        }

        RefTable::read(&mut r)?;

        if header.body_compression == Compression::Compressed {
            let body_size = r.u32()?;
            let compressed_body_size = r.u32()?;
            let compressed_body = r.bytes(compressed_body_size as usize)?;
            let mut body = vec![0; body_size as usize];

            minilzo::decompress_to_slice(&compressed_body, &mut body).unwrap();

            let mut r = Reader::with_id_and_node_state(
                Cursor::new(body.as_slice()),
                reader::IdState::new(),
                reader::NodeState::new(header.num_nodes as usize),
            );

            r.chunk_id(0x0304300D)?;
            r.optional_id()?;
            r.u32()?;
            r.u32()?;

            r.chunk_id(0x03043011)?;
            r.node(0x0301B000, |r| {
                r.chunk_id(0x0301B000)?;
                if r.bool()? {
                    r.id()?;
                    r.u32()?;
                    r.id()?;
                    r.u32()?;
                }

                r.node_end()?;

                Ok(())
            })?;
            r.node(0x0305B000, |r| {
                r.chunk_id(0x0305B001)?;
                r.u32()?;
                r.u32()?;
                r.u32()?;
                r.u32()?;

                r.chunk_id(0x0305B004)?;
                map.bronze_time = r.u32()?;
                map.silver_time = r.u32()?;
                map.gold_time = r.u32()?;
                map.author_time = r.u32()?;
                let _author_score = r.u32()?;

                r.chunk_id(0x0305B008)?;
                r.u32()?;
                r.u32()?;

                r.skip_chunk(0x0305B00A)?;

                r.chunk_id(0x0305B00D)?;
                let _validation_ghost = r.optional_node(0x03092000, Ghost::read)?;

                r.skippable_chunk_id(0x0305B00E)?;
                let _map_type = r.string()?;
                let _map_style = r.string()?;
                map.is_validated = r.bool()?;

                r.node_end()?;

                Ok(())
            })?;
            let _map_kind = r.u32()?;

            r.skippable_chunk_id(0x03043018)?;
            if r.bool()? {
                map.num_laps = Some(r.u32()?)
            } else {
                r.skip(4)?;
            }

            r.skippable_chunk_id(0x03043019)?;
            map.texture_mod = r.optional_file_ref()?;

            r.chunk_id(0x0304301F)?;
            r.id()?;
            r.u32()?;
            r.id()?;
            map.name = r.string()?;
            map.deco_name = r.id()?;
            r.u32()?;
            let _deco_author = r.id()?;
            map.size.x = r.u32()?;
            map.size.y = r.u32()?;
            map.size.z = r.u32()?;
            r.u32()?;
            r.u32()?;
            let num_blocks = r.u32()?;
            let mut blocks = Vec::with_capacity(num_blocks as usize);
            while r.peek_u32()? & 0x4FFFF000 == 0x40000000 {
                let id = r.id()?;
                let dir = Direction::try_from(r.u8()?).unwrap();
                let x = r.u8()?;
                let y = r.u8()?;
                let z = r.u8()?;
                let flags = r.u32()?;

                if flags == 0xFFFFFFFF {
                    continue;
                }

                let is_ground = flags & 0x00001000 != 0;

                let skin = if flags & 0x00008000 != 0 {
                    let _author = r.id()?; // "dsTdptYAS06hYsbqyCZi1A"
                    r.optional_node_owned(0x03059000, Skin::read)?
                } else {
                    None
                };

                let waypoint_property = if flags & 0x00100000 != 0 {
                    Some(r.node_owned(0x2E009000, WaypointProperty::read)?)
                } else {
                    None
                };

                let variant_index = if flags & 0x00200000 != 0 { 1 } else { 0 };

                let is_ghost = flags & 0x10000000 != 0;

                let block_type = if flags & 0x20000000 != 0 {
                    BlockType::Free(FreeBlock {
                        id,
                        skin,
                        waypoint_property,
                        ..Default::default()
                    })
                } else {
                    BlockType::Normal(Block {
                        id,
                        dir,
                        coord: Vec3 { x, y, z },
                        is_ground,
                        skin,
                        waypoint_property,
                        variant_index,
                        is_ghost,
                        ..Default::default()
                    })
                };

                blocks.push(block_type);
            }

            r.chunk_id(0x03043022)?;
            r.u32()?;

            r.chunk_id(0x03043024)?;
            map.music = r.optional_file_ref()?;

            r.chunk_id(0x03043025)?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            r.chunk_id(0x03043026)?;
            r.u32()?;

            r.chunk_id(0x03043028)?;
            r.u32()?;
            r.string()?;

            r.skip_chunk(0x03043029)?;

            r.chunk_id(0x0304302A)?;
            r.u32()?;

            r.skip_chunk(0x03043034)?;
            r.skip_chunk(0x03043036)?;
            r.skip_chunk(0x03043038)?;
            r.skip_chunk(0x0304303E)?;

            r.skippable_chunk_id(0x03043040)?;
            r.u32()?;
            r.u32()?;
            let size = r.u32()?;
            let bytes = r.bytes(size as usize)?;
            {
                let mut r = Reader::with_id_state(Cursor::new(bytes), reader::IdState::new());
                r.u32()?;
                map.items = r.list(|r| r.flat_node(0x03101000, Item::read))?;
                r.u32()?; // 0
                r.u32()?; // 0
                r.u32()?; // 0
                r.u32()?; // 0
                r.u32()?; // 0
                r.u32()?; // 0
            }

            r.skippable_chunk_id(0x03043042)?;
            r.u32()?;
            r.u32()?;
            let _author_login = r.string()?;
            map.author_name = r.string()?;
            map.author_zone = r.string()?;
            r.u32()?;

            r.skip_chunk(0x03043043)?;
            r.skip_chunk(0x03043044)?;

            r.skippable_chunk_id(0x03043048)?;
            r.u32()?;
            r.u32()?;
            let num_baked_blocks = r.u32()?;
            map.baked_blocks = Vec::with_capacity(num_baked_blocks as usize);
            while r.peek_u32()? & 0x4FFFF000 == 0x40000000 {
                let id = r.id()?;
                let dir = Direction::try_from(r.u8()?).unwrap();
                let x = r.u8()?;
                let y = r.u8()?;
                let z = r.u8()?;
                let flags = r.u32()?;

                if flags == 0xFFFFFFFF {
                    continue;
                }

                if flags & 0x00008000 != 0 {
                    r.id()?;
                    r.u32()?;
                }

                let is_ghost = flags & 0x10000000 != 0;

                let block_type = if flags & 0x20000000 != 0 {
                    BlockType::Free(FreeBlock {
                        id,
                        ..Default::default()
                    })
                } else {
                    BlockType::Normal(Block {
                        id,
                        dir,
                        coord: Vec3 { x, y, z },
                        is_ghost,
                        ..Default::default()
                    })
                };

                map.baked_blocks.push(block_type);
            }
            r.u32()?;
            r.u32()?;

            r.chunk_id(0x03043049)?;
            r.u32()?;
            map.intro_media = r.optional_node_owned(0x03079000, media::Clip::read)?;
            map.podium_media = r.optional_node_owned(0x03079000, media::Clip::read)?;
            map.in_game_media = r.optional_node_owned(0x0307A000, media::ClipGroup::read)?;
            map.end_race_media = r.optional_node_owned(0x0307A000, media::ClipGroup::read)?;
            map.ambiance_media = r.optional_node_owned(0x03079000, media::Clip::read)?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            r.skip_chunk(0x0304304B)?;
            r.skip_chunk(0x0304304F)?;
            r.skip_chunk(0x03043050)?;
            r.skip_chunk(0x03043051)?;
            r.skip_chunk(0x03043052)?;
            r.skip_chunk(0x03043053)?;

            r.skippable_chunk_id(0x03043054)?;
            r.u32()?; // 1
            r.u32()?; // 0
            let size = r.u32()?;
            let bytes = r.bytes(size as usize)?;

            {
                let mut r = Reader::with_id_state(bytes.as_slice(), reader::IdState::new());
                map.embedded_file_ids = r.list(|r| {
                    let id = r.id()?;
                    r.u32()?; // 26
                    r.optional_id()?; // "pTuyJG9STcCN_11BiU3t0Q"

                    Ok(id)
                })?;
                let size = r.u32()?;
                if size > 0 {
                    let bytes = r.bytes(size as usize)?;
                    map.embedded_files = bytes;
                }
                r.u32()?; // 0
            }

            r.skip_chunk(0x03043055)?;

            r.skippable_chunk_id(0x03043056)?;
            r.u32()?;
            r.u32()?;
            let _time_of_day = r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            r.skip_optional_chunk(0x03043057)?;
            r.skip_optional_chunk(0x03043058)?;
            r.skip_chunk(0x03043059)?;
            r.skip_chunk(0x0304305A)?;
            r.skip_chunk(0x0304305B)?;
            r.skip_chunk(0x0304305C)?;
            r.skip_chunk(0x0304305D)?;
            r.skip_chunk(0x0304305E)?;

            r.skippable_chunk_id(0x0304305F)?;
            r.u32()?;
            for block in &mut blocks {
                if let BlockType::Free(free_block) = block {
                    let x = r.f32()?;
                    let y = r.f32()?;
                    let z = r.f32()?;
                    let yaw = r.f32()?;
                    let pitch = r.f32()?;
                    let roll = r.f32()?;

                    free_block.pos = Vec3 { x, y, z };
                    free_block.yaw = yaw;
                    free_block.pitch = pitch;
                    free_block.roll = roll;
                }
            }
            for baked_block in &mut map.baked_blocks {
                if let BlockType::Free(free_block) = baked_block {
                    let x = r.f32()?;
                    let y = r.f32()?;
                    let z = r.f32()?;
                    let yaw = r.f32()?;
                    let pitch = r.f32()?;
                    let roll = r.f32()?;

                    free_block.pos = Vec3 { x, y, z };
                    free_block.yaw = yaw;
                    free_block.pitch = pitch;
                    free_block.roll = roll;
                }
            }

            r.skip_optional_chunk(0x03043060)?;
            r.skip_optional_chunk(0x03043061)?;

            r.optional_skippable_chunk(0x03043062, |r| {
                r.u32()?;
                for block in &mut blocks {
                    match block {
                        BlockType::Normal(block) => block.color = Color::try_from(r.u8()?).unwrap(),
                        BlockType::Free(free_block) => {
                            free_block.color = Color::try_from(r.u8()?).unwrap()
                        }
                    }
                }
                for baked_block in &mut map.baked_blocks {
                    match baked_block {
                        BlockType::Normal(block) => block.color = Color::try_from(r.u8()?).unwrap(),
                        BlockType::Free(free_block) => {
                            free_block.color = Color::try_from(r.u8()?).unwrap()
                        }
                    }
                }
                for item in &mut map.items {
                    item.color = Color::try_from(r.u8()?).unwrap();
                }

                Ok(())
            })?;

            r.optional_skippable_chunk(0x03043063, |r| {
                r.u32()?;
                for item in &mut map.items {
                    item.anim_offset = PhaseOffset::try_from(r.u8()?).unwrap()
                }

                Ok(())
            })?;

            r.skip_optional_chunk(0x03043064)?;
            r.skip_optional_chunk(0x03043065)?;
            r.skip_optional_chunk(0x03043067)?;

            r.optional_skippable_chunk(0x03043068, |r| {
                r.u32()?;
                for block in &mut blocks {
                    match block {
                        BlockType::Normal(block) => {
                            block.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap()
                        }
                        BlockType::Free(free_block) => {
                            free_block.lightmap_quality =
                                LightmapQuality::try_from(r.u8()?).unwrap()
                        }
                    }
                }
                for baked_block in &mut map.baked_blocks {
                    match baked_block {
                        BlockType::Normal(block) => {
                            block.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap()
                        }
                        BlockType::Free(free_block) => {
                            free_block.lightmap_quality =
                                LightmapQuality::try_from(r.u8()?).unwrap()
                        }
                    }
                }
                for item in &mut map.items {
                    item.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap();
                }

                Ok(())
            })?;

            r.skip_optional_chunk(0x03043069)?;

            r.node_end()?;

            for block in blocks {
                match block {
                    BlockType::Normal(block) => map.blocks.push(block),
                    BlockType::Free(free_block) => map.free_blocks.push(free_block),
                }
            }
        } else {
            todo!()
        }

        Ok(map)
    }

    // Read a map from a file at the given `path`.
    pub fn read_from_file<P>(path: P) -> ReadResult<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::read_from(reader)
    }

    /// Write the map to the given `writer`.
    ///
    /// For performance reasons, it is recommended that the `writer` is buffered.
    pub fn write_to<W>(&self, writer: W) -> WriteResult
    where
        W: Write,
    {
        let map_type = "TrackMania\\TM_Race";
        let lightmap_version = 8;
        let title = "TMStadium";

        let mut xml = vec![];
        quick_xml::Writer::new(&mut xml)
            .create_element("header")
            .with_attribute(("exever", "3.3.0"))
            .with_attribute(("exebuild", "2022-07-06_11_37"))
            .with_attribute(("title", title))
            .with_attribute(("lightmap", lightmap_version.to_string().as_str()))
            .write_inner_content(|w| {
                w.create_element("ident")
                    .with_attribute(("uid", ""))
                    .with_attribute(("name", self.name.as_str()))
                    .with_attribute(("author", ""))
                    .with_attribute(("authorzone", self.author_zone.as_str()))
                    .write_empty()?;
                w.create_element("desc")
                    .with_attribute(("envir", "Stadium"))
                    .with_attribute(("mood", "Day"))
                    .with_attribute(("type", "Race"))
                    .with_attribute(("maptype", map_type))
                    .with_attribute(("mapstyle", ""))
                    .with_attribute(("validated", if self.is_validated { "1" } else { "0" }))
                    .with_attribute(("nblaps", self.num_laps.unwrap_or(0).to_string().as_str()))
                    .with_attribute(("displaycost", self.cost.to_string().as_str()))
                    .with_attribute(("mod", ""))
                    .with_attribute(("hasghostblocks", "1"))
                    .write_empty()?;
                w.create_element("playermodel")
                    .with_attribute(("id", ""))
                    .write_empty()?;
                w.create_element("times")
                    .with_attribute(("bronze", self.bronze_time.to_string().as_str()))
                    .with_attribute(("silver", self.silver_time.to_string().as_str()))
                    .with_attribute(("gold", self.gold_time.to_string().as_str()))
                    .with_attribute(("authortime", self.author_time.to_string().as_str()))
                    .with_attribute(("authorscore", "0"))
                    .write_empty()?;
                w.create_element("deps").write_inner_content(|_| Ok(()))?;
                Ok(())
            })
            .unwrap();

        let mut items = vec![];
        let mut w = Writer::with_id_state(&mut items, writer::IdState::new());
        w.u32(10)?;
        w.u32(self.items.len() as u32)?;
        for item in &self.items {
            w.flat_node(0x03101000, |w| item.write(w))?;
        }
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;

        let mut generalogies = vec![];
        let mut w = Writer::new(&mut generalogies);
        w.u32(0)?;

        let mut metadata = vec![];
        let mut w = Writer::new(&mut metadata);
        w.flat_node(0x11002000, |w| {
            w.u32(6)?;
            w.u8(2)?;
            w.u8(2)?;
            w.u8(7)?;
            w.u8(0)?;
            w.u8(2)?;
            w.u8(2)?;
            w.u8(25)?;
            w.bytes(b"LibMapType_MapTypeVersion")?;
            w.u8(0)?;
            w.u32(1)?;
            w.u8(28)?;
            w.bytes(b"Race_AuthorRaceWaypointTimes")?;
            w.u8(1)?;
            w.u8(0)?;

            Ok(())
        })?;

        let mut chunk2 = vec![];
        let mut w = Writer::new(&mut chunk2);
        w.u8(13)?;
        w.u32(0)?;
        w.u32(self.bronze_time)?;
        w.u32(self.silver_time)?;
        w.u32(self.gold_time)?;
        w.u32(self.author_time)?;
        w.u32(self.cost)?;
        w.bool(self.num_laps.is_some())?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(2)?;
        w.u32(0)?;
        w.u32(self.num_cps)?;
        w.u32(self.num_laps.unwrap_or(1))?;

        let mut chunk3 = vec![];
        let mut w = Writer::with_id_state(&mut chunk3, writer::IdState::new());
        w.u8(11)?;
        w.null_id()?; // some ubi id
        w.u32(26)?;
        w.null_id()?; // some ubi id
        w.string(&self.name)?;
        w.u8(8)?;
        w.u32(0)?;
        w.u32(0)?;
        w.id(&self.deco_name)?;
        w.u32(26)?;
        w.id("Nadeo")?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.string(map_type)?;
        w.u32(0)?;
        w.u64(0)?; // some uid
        w.u8(lightmap_version)?;
        w.id(title)?;

        let mut chunk4 = vec![];
        let mut w = Writer::new(&mut chunk4);
        w.u32(6)?;

        let mut chunk5 = vec![];
        let mut w = Writer::new(&mut chunk5);
        w.u32(xml.len() as u32)?;
        w.bytes(&xml)?;

        let mut chunk7 = vec![];
        let mut w = Writer::new(&mut chunk7);
        if let Some(thumbnail) = &self.thumbnail {
            w.bool(true)?;
            w.u32(thumbnail.len() as u32)?;
            w.bytes(b"<Thumbnail.jpg>")?;
            w.bytes(thumbnail)?;
            w.bytes(b"</Thumbnail.jpg>")?;
            w.bytes(b"<Comments>")?;
            w.string("")?;
            w.bytes(b"</Comments>")?;
        } else {
            w.bool(false)?;
        }

        let mut chunk8 = vec![];
        let mut w = Writer::new(&mut chunk8);
        w.u32(1)?;
        w.u32(0)?;
        w.string("")?; // ubi uid
        w.string(&self.author_name)?;
        w.string(&self.author_zone)?;
        w.u32(0)?;

        let mut user_data = vec![];
        let mut w = Writer::new(&mut user_data);
        w.u32(6)?;
        w.u32(0x03043002)?;
        w.u32(chunk2.len() as u32)?;
        w.u32(0x03043003)?;
        w.u32(chunk3.len() as u32)?;
        w.u32(0x03043004)?;
        w.u32(chunk4.len() as u32)?;
        w.u32(0x03043005)?;
        w.u32(chunk5.len() as u32)?;
        w.u32(0x03043007)?;
        w.u32(chunk7.len() as u32)?;
        w.u32(0x03043008)?;
        w.u32(chunk8.len() as u32)?;
        w.bytes(&chunk2)?;
        w.bytes(&chunk3)?;
        w.bytes(&chunk4)?;
        w.bytes(&chunk5)?;
        w.bytes(&chunk7)?;
        w.bytes(&chunk8)?;

        let mut body = vec![];
        let mut w = Writer::with_id_and_node_state(
            &mut body,
            writer::IdState::new(),
            writer::NodeState::new(),
        );

        w.chunk_id(0x0304300D)?;
        w.null_id()?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;

        w.chunk_id(0x03043011)?;
        w.node(0x0301B000, |w| {
            w.chunk_id(0x0301B000)?;
            w.bool(false)?;

            Ok(())
        })?;
        w.node(0x0305B000, |w| {
            w.chunk_id(0x0305B001)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            w.chunk_id(0x0305B004)?;
            w.u32(self.bronze_time)?;
            w.u32(self.silver_time)?;
            w.u32(self.gold_time)?;
            w.u32(self.author_time)?;
            w.u32(0)?;

            w.chunk_id(0x0305B008)?;
            w.u32(60000)?;
            w.u32(0)?;

            w.skippable_chunk(0x0305B00A, |w| {
                w.u32(0)?;
                w.u32(self.bronze_time)?;
                w.u32(self.silver_time)?;
                w.u32(self.gold_time)?;
                w.u32(self.author_time)?;
                w.u32(60000)?;
                w.u32(0)?;

                Ok(())
            })?;

            w.chunk_id(0x0305B00D)?;
            w.null_node()?;

            w.skippable_chunk(0x0305B00E, |w| {
                w.string(map_type)?;
                w.u32(0)?;
                w.bool(self.is_validated)?;

                Ok(())
            })?;

            Ok(())
        })?;
        w.u32(8)?;

        w.skippable_chunk(0x03043018, |w| {
            w.bool(self.num_laps.is_some())?;
            w.u32(self.num_laps.unwrap_or(3))?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043019, |w| {
            if let Some(texture_mod) = &self.texture_mod {
                w.file_ref(texture_mod)?;
            } else {
                w.null_file_ref()?;
            }

            Ok(())
        })?;

        w.chunk_id(0x0304301F)?;
        w.null_id()?;
        w.u32(26)?;
        w.null_id()?;
        w.string(&self.name)?;
        w.id(&self.deco_name)?;
        w.u32(26)?;
        w.id("Nadeo")?;
        w.u32(self.size.x)?;
        w.u32(self.size.y)?;
        w.u32(self.size.z)?;
        w.u32(0)?;
        w.u32(6)?;
        w.u32(self.blocks.len() as u32 + self.free_blocks.len() as u32)?;
        for block in &self.blocks {
            let mut flags = 0x00000000;

            if block.is_ground {
                flags &= 0x00001000;
            }

            if block.skin.is_some() {
                flags &= 0x00008000;
            }

            if block.waypoint_property.is_some() {
                flags &= 0x00100000;
            }

            if block.variant_index == 1 {
                flags &= 0x00200000;
            }

            if block.is_ghost {
                flags &= 0x10000000;
            }

            w.id(&block.id)?;
            w.u8(block.dir.into())?;
            w.u8(block.coord.x)?;
            w.u8(block.coord.y)?;
            w.u8(block.coord.z)?;
            w.u32(flags)?;
            if let Some(skin) = &block.skin {
                w.null_id()?;
                w.node(0x03059000, |w| skin.write(w))?;
            }
            if let Some(waypoint_property) = &block.waypoint_property {
                w.node(0x2E009000, |w| waypoint_property.write(w))?;
            }
        }
        for free_block in &self.free_blocks {
            let mut flags = 0x20000000;

            if free_block.skin.is_some() {
                flags &= 0x00008000;
            }

            if free_block.waypoint_property.is_some() {
                flags &= 0x00100000;
            }

            w.id(&free_block.id)?;
            w.u8(0)?;
            w.u8(0)?;
            w.u8(0)?;
            w.u8(0)?;
            w.u32(flags)?;
            if let Some(skin) = &free_block.skin {
                w.null_id()?;
                w.node(0x03059000, |w| skin.write(w))?;
            }
            if let Some(waypoint_property) = &free_block.waypoint_property {
                w.node(0x2E009000, |w| waypoint_property.write(w))?;
            }
        }

        w.chunk_id(0x03043022)?;
        w.u32(1)?;

        w.chunk_id(0x03043024)?;
        if let Some(music) = &self.music {
            w.file_ref(music)?;
        } else {
            w.null_file_ref()?;
        }

        w.chunk_id(0x03043025)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;

        w.chunk_id(0x03043026)?;
        w.u32(0xFFFFFFFF)?;

        w.chunk_id(0x03043028)?;
        w.u32(0)?;
        w.string("")?;

        w.skippable_chunk(0x03043029, |w| {
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.chunk_id(0x0304302A)?;
        w.u32(0)?;

        w.skippable_chunk(0x03043034, |w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043036, |w| {
            w.f32(0.0)?;
            w.f32(0.0)?;
            w.f32(0.0)?;
            w.f32(0.0)?;
            w.f32(0.0)?;
            w.f32(0.0)?;
            w.f32(90.0)?;
            w.f32(10.0)?;
            w.f32(0.0)?;
            w.f32(-1.0)?;
            w.f32(-1.0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043038, |w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304303E, |w| {
            w.u32(0)?;
            w.u32(10)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043040, |w| {
            w.u32(7)?;
            w.u32(0)?;
            w.u32(items.len() as u32)?;
            w.bytes(&items)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043042, |w| {
            w.u32(1)?;
            w.u32(0)?;
            w.string("")?;
            w.string(&self.author_name)?;
            w.string(&self.author_zone)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043043, |w| {
            w.u32(0)?;
            w.u32(generalogies.len() as u32)?;
            w.bytes(&generalogies)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043044, |w| {
            w.u32(0)?;
            w.u32(metadata.len() as u32)?;
            w.bytes(&metadata)?;

            Ok(())
        })?;

        let num_nodes = w.num_nodes();

        let mut w = Writer::new(writer);
        Header::write(&mut w, 0x03043000, &user_data, num_nodes)?;
        RefTable::write(&mut w)?;

        let mut compressed_body = vec![];
        minilzo::compress(&body, &mut compressed_body);

        w.u32(body.len() as u32)?;
        w.u32(compressed_body.len() as u32)?;
        w.bytes(&compressed_body)?;

        Ok(())
    }

    /// Write the map to a file at the given `path`.
    pub fn write_to_file<P>(&self, path: P) -> WriteResult
    where
        P: AsRef<Path>,
    {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        self.write_to(writer)
    }
}
