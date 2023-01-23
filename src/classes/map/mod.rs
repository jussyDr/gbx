/// Media tracker types.
pub mod media;

use crate::error::ReadResult;
use crate::gbx::{Class, ReadBody, ReadChunk, ReadChunkFn, ReadHeader};
use crate::reader::{self, Reader};
use crate::types::{RcStr, Vec3};
use crate::{gbx, FileRef, Ghost};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
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

/// Color of a block or item.
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

/// Lightmap quality of a block or item.
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

/// Skin of a block or item.
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

/// Order of a start, finish or multilap block or item in royal.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, TryFromPrimitive, IntoPrimitive)]
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

/// Waypoint property of a block or item.
#[derive(Clone, Default, Debug)]
#[non_exhaustive]
pub enum WaypointProperty {
    /// Checkpoint.
    #[default]
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
        let mut waypoint_property = Self::default();
        gbx::read_body(&mut waypoint_property, r)?;
        Ok(waypoint_property)
    }

    fn read_chunk_2e009000<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?; // 2
        let tag = r.string()?;
        *self = match tag.as_str() {
            "Checkpoint" => {
                r.u32()?;
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

        Ok(())
    }
}

impl<R, I, N> ReadBody<R, I, N> for WaypointProperty
where
    R: Read,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x2E009000, ReadChunk::Read(Self::read_chunk_2e009000)),
            (0x2E009001, ReadChunk::Skip),
        ]
    }
}

/// A block inside of a `Map`.
#[derive(Default)]
pub struct Block {
    /// Id of the block's model.
    pub model_id: RcStr,
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
    pub model_id: RcStr,
    /// Skin of the block, e.g. for signs.
    pub skin: Option<Skin>,
    /// Waypoint property.
    pub waypoint_property: Option<WaypointProperty>,
    /// Absolute position of the block.
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

/// Either a 'normal' or free block.
pub enum BlockType {
    /// A 'normal' block.
    Normal(Block),
    /// A free block.
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
    pub model_id: RcStr,
    /// Yaw rotation of the item.
    pub yaw: f32,
    /// Pitch rotation of the item.
    pub pitch: f32,
    /// Roll rotation of the item.
    pub roll: f32,
    /// Coord inside the map.
    pub coord: Vec3<u8>,
    /// Absolute position inside the map.
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
        let mut item = Self::default();
        gbx::read_body(&mut item, r)?;
        Ok(item)
    }

    fn read_chunk_03101002<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
    {
        r.u32()?; // 8
        self.model_id = r.id()?;
        r.u32()?; // 26
        let _author = r.optional_id()?; // "Nadeo"
        self.yaw = r.f32()?;
        self.pitch = r.f32()?;
        self.roll = r.f32()?;
        self.coord.x = r.u8()?;
        self.coord.y = r.u8()?;
        self.coord.z = r.u8()?;
        r.u32()?; // 0xFFFFFFFF
        self.pos.x = r.f32()?;
        self.pos.y = r.f32()?;
        self.pos.z = r.f32()?;
        self.waypoint_property = r.optional_flat_node(0x2E009000, WaypointProperty::read)?;
        let flags = r.u16()?;
        self.pivot_pos.x = r.f32()?;
        self.pivot_pos.y = r.f32()?;
        self.pivot_pos.z = r.f32()?;
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

        Ok(())
    }
}

impl<R, I, N> ReadBody<R, I, N> for Item
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x03101002, ReadChunk::Read(Self::read_chunk_03101002)),
            (0x03101004, ReadChunk::Skip),
            (0x03101005, ReadChunk::Skip),
        ]
    }
}

/// Type corresponding to the file extension `Map.Gbx`.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> gbx::error::ReadResult<()> {
/// let map = gbx::Map::read_from_file("MyMap.Map.Gbx")?;
/// # Ok(())
/// # }
/// ```
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
    /// Number of laps if the map is multilap.
    pub num_laps: Option<u32>,
    /// Name of the map.
    pub name: String,
    /// Id of the map decoration.
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
    /// All (free) blocks placed inside of the map.
    pub blocks: Vec<BlockType>,
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
    pub embedded_files: Option<Vec<u8>>,

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
        gbx::read(reader)
    }

    fn read_chunk_03043002<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u8()?;
        r.u32()?;
        self.bronze_time = r.u32()?;
        self.silver_time = r.u32()?;
        self.gold_time = r.u32()?;
        self.author_time = r.u32()?;
        self.cost = r.u32()?;
        let is_multilap = r.bool()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        self.num_cps = r.u32()?;
        if is_multilap {
            self.num_laps = Some(r.u32()?)
        } else {
            r.u32()?;
        }

        Ok(())
    }

    fn read_chunk_03043003<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        r.u8()?;
        r.id()?;
        r.u32()?;
        r.id()?;
        self.name = r.string()?;
        r.u8()?;
        r.u32()?;
        r.u32()?;
        self.deco_name = r.id()?;
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

        Ok(())
    }

    fn read_chunk_03043004<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let _version = r.u32()?;

        Ok(())
    }

    fn read_chunk_03043005<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let _xml = r.string()?;

        Ok(())
    }

    fn read_chunk_03043007<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        if r.bool()? {
            let thumbnail_size = r.u32()?;
            r.bytes(15)?;
            self.thumbnail = Some(r.bytes(thumbnail_size as usize)?);
            r.bytes(16)?;
            r.bytes(10)?;
            let _map_comments = r.string()?;
            r.bytes(11)?;
        }

        Ok(())
    }

    fn read_chunk_03043008<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        let _author_login = r.string()?;
        self.author_name = r.string()?;
        self.author_zone = r.string()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_0304300d<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        r.optional_id()?;
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043011<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
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
            self.bronze_time = r.u32()?;
            self.silver_time = r.u32()?;
            self.gold_time = r.u32()?;
            self.author_time = r.u32()?;
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
            self.is_validated = r.bool()?;

            r.node_end()?;

            Ok(())
        })?;
        let _map_kind = r.u32()?;

        Ok(())
    }

    fn read_chunk_03043018<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        if r.bool()? {
            self.num_laps = Some(r.u32()?)
        } else {
            r.u32()?;
        }

        Ok(())
    }

    fn read_chunk_03043019<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        self.texture_mod = r.optional_file_ref()?;

        Ok(())
    }

    fn read_chunk_0304301f<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        r.id()?;
        r.u32()?;
        r.id()?;
        self.name = r.string()?;
        self.deco_name = r.id()?;
        r.u32()?;
        let _deco_author = r.id()?;
        self.size.x = r.u32()?;
        self.size.y = r.u32()?;
        self.size.z = r.u32()?;
        r.u32()?;
        r.u32()?;
        let num_blocks = r.u32()?;
        self.blocks = Vec::with_capacity(num_blocks as usize);
        while r.peek_u32()? & 0x4FFFF000 == 0x40000000 {
            let model_id = r.id()?;
            let dir = Direction::try_from(r.u8()?).unwrap();
            let coord = r.vec3u8()?;
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
                    model_id,
                    skin,
                    waypoint_property,
                    ..Default::default()
                })
            } else {
                BlockType::Normal(Block {
                    model_id,
                    dir,
                    coord,
                    is_ground,
                    skin,
                    waypoint_property,
                    variant_index,
                    is_ghost,
                    ..Default::default()
                })
            };

            self.blocks.push(block_type);
        }

        Ok(())
    }

    fn read_chunk_03043022<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043024<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        self.music = r.optional_file_ref()?;

        Ok(())
    }

    fn read_chunk_03043025<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043026<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043028<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.string()?;

        Ok(())
    }

    fn read_chunk_0304302a<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043040<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        let size = r.u32()?;
        let bytes = r.bytes(size as usize)?;
        {
            let mut r = Reader::with_id_state(Cursor::new(bytes), reader::IdState::new());
            r.u32()?;
            self.items = r.list(|r| r.flat_node(0x03101000, Item::read))?;
            r.list(|r| r.u32())?;
            r.list(|r| r.u32())?;
            r.list(|r| r.u32())?;
        }

        Ok(())
    }

    fn read_chunk_03043042<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        let _author_login = r.string()?;
        self.author_name = r.string()?;
        self.author_zone = r.string()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043048<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
    {
        r.u32()?;
        r.u32()?;
        let num_baked_blocks = r.u32()?;
        self.baked_blocks = Vec::with_capacity(num_baked_blocks as usize);
        while r.peek_u32()? & 0x4FFFF000 == 0x40000000 {
            let model_id = r.id()?;
            let dir = Direction::try_from(r.u8()?).unwrap();
            let coord = r.vec3u8()?;
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
                    model_id,
                    ..Default::default()
                })
            } else {
                BlockType::Normal(Block {
                    model_id,
                    dir,
                    coord,
                    is_ghost,
                    ..Default::default()
                })
            };

            self.baked_blocks.push(block_type);
        }
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043049<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        r.u32()?;
        self.intro_media = r.optional_node_owned(0x03079000, media::Clip::read)?;
        self.podium_media = r.optional_node_owned(0x03079000, media::Clip::read)?;
        self.in_game_media = r.optional_node_owned(0x0307A000, media::ClipGroup::read)?;
        self.end_race_media = r.optional_node_owned(0x0307A000, media::ClipGroup::read)?;
        self.ambiance_media = r.optional_node_owned(0x03079000, media::Clip::read)?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_03043054<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?; // 1
        r.u32()?; // 0
        let size = r.u32()?;
        let bytes = r.bytes(size as usize)?;
        {
            let mut r = Reader::with_id_state(bytes.as_slice(), reader::IdState::new());
            self.embedded_file_ids = r.list(|r| {
                let id = r.id()?;
                r.u32()?; // 26
                r.optional_id()?; // "pTuyJG9STcCN_11BiU3t0Q"

                Ok(id)
            })?;
            let size = r.u32()?;
            if size > 0 {
                let bytes = r.bytes(size as usize)?;
                self.embedded_files = Some(bytes);
            }
            r.u32()?; // 0
        }

        Ok(())
    }

    fn read_chunk_03043056<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        r.u32()?;
        let _time_of_day = r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        Ok(())
    }

    fn read_chunk_0304305f<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        for block in &mut self.blocks {
            if let BlockType::Free(free_block) = block {
                free_block.pos = r.vec3f32()?;
                free_block.yaw = r.f32()?;
                free_block.pitch = r.f32()?;
                free_block.roll = r.f32()?;
            }
        }
        for baked_block in &mut self.baked_blocks {
            if let BlockType::Free(free_block) = baked_block {
                free_block.pos = r.vec3f32()?;
                free_block.yaw = r.f32()?;
                free_block.pitch = r.f32()?;
                free_block.roll = r.f32()?;
            }
        }

        Ok(())
    }

    fn read_chunk_03043062<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        for block in &mut self.blocks {
            match block {
                BlockType::Normal(block) => block.color = Color::try_from(r.u8()?).unwrap(),
                BlockType::Free(free_block) => free_block.color = Color::try_from(r.u8()?).unwrap(),
            }
        }
        for baked_block in &mut self.baked_blocks {
            match baked_block {
                BlockType::Normal(block) => block.color = Color::try_from(r.u8()?).unwrap(),
                BlockType::Free(free_block) => free_block.color = Color::try_from(r.u8()?).unwrap(),
            }
        }
        for item in &mut self.items {
            item.color = Color::try_from(r.u8()?).unwrap();
        }

        Ok(())
    }

    fn read_chunk_03043063<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        for item in &mut self.items {
            item.anim_offset = PhaseOffset::try_from(r.u8()?).unwrap()
        }

        Ok(())
    }

    fn read_chunk_03043068<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u32()?;
        for block in &mut self.blocks {
            match block {
                BlockType::Normal(block) => {
                    block.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap()
                }
                BlockType::Free(free_block) => {
                    free_block.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap()
                }
            }
        }
        for baked_block in &mut self.baked_blocks {
            match baked_block {
                BlockType::Normal(block) => {
                    block.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap()
                }
                BlockType::Free(free_block) => {
                    free_block.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap()
                }
            }
        }
        for item in &mut self.items {
            item.lightmap_quality = LightmapQuality::try_from(r.u8()?).unwrap();
        }

        Ok(())
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
}

impl Class for Map {
    const CLASS_ID: u32 = 0x03043000;
}

impl<R, I, N> ReadHeader<R, I, N> for Map
where
    R: Read,
    I: BorrowMut<reader::IdState>,
{
    fn header_chunks<'a>() -> &'a [(u32, ReadChunkFn<Self, R, I, N>)] {
        &[
            (0x03043002, Self::read_chunk_03043002),
            (0x03043003, Self::read_chunk_03043003),
            (0x03043004, Self::read_chunk_03043004),
            (0x03043005, Self::read_chunk_03043005),
            (0x03043007, Self::read_chunk_03043007),
            (0x03043008, Self::read_chunk_03043008),
        ]
    }
}

impl<R, I, N> ReadBody<R, I, N> for Map
where
    R: Read + Seek,
    I: BorrowMut<reader::IdState>,
    N: BorrowMut<reader::NodeState>,
{
    fn body_chunks<'a>() -> &'a [(u32, ReadChunk<Self, R, I, N>)] {
        &[
            (0x0304300D, ReadChunk::Read(Self::read_chunk_0304300d)),
            (0x03043011, ReadChunk::Read(Self::read_chunk_03043011)),
            (
                0x03043018,
                ReadChunk::ReadSkippable(Self::read_chunk_03043018),
            ),
            (
                0x03043019,
                ReadChunk::ReadSkippable(Self::read_chunk_03043019),
            ),
            (0x0304301F, ReadChunk::Read(Self::read_chunk_0304301f)),
            (0x03043022, ReadChunk::Read(Self::read_chunk_03043022)),
            (0x03043024, ReadChunk::Read(Self::read_chunk_03043024)),
            (0x03043025, ReadChunk::Read(Self::read_chunk_03043025)),
            (0x03043026, ReadChunk::Read(Self::read_chunk_03043026)),
            (0x03043028, ReadChunk::Read(Self::read_chunk_03043028)),
            (0x03043029, ReadChunk::Skip),
            (0x0304302A, ReadChunk::Read(Self::read_chunk_0304302a)),
            (0x03043034, ReadChunk::Skip),
            (0x03043036, ReadChunk::Skip),
            (0x03043038, ReadChunk::Skip),
            (0x0304303E, ReadChunk::Skip),
            (
                0x03043040,
                ReadChunk::ReadSkippable(Self::read_chunk_03043040),
            ),
            (
                0x03043042,
                ReadChunk::ReadSkippable(Self::read_chunk_03043042),
            ),
            (0x03043043, ReadChunk::Skip),
            (0x03043044, ReadChunk::Skip),
            (
                0x03043048,
                ReadChunk::ReadSkippable(Self::read_chunk_03043048),
            ),
            (0x03043049, ReadChunk::Read(Self::read_chunk_03043049)),
            (0x0304304B, ReadChunk::Skip),
            (0x0304304F, ReadChunk::Skip),
            (0x03043050, ReadChunk::Skip),
            (0x03043051, ReadChunk::Skip),
            (0x03043052, ReadChunk::Skip),
            (0x03043053, ReadChunk::Skip),
            (
                0x03043054,
                ReadChunk::ReadSkippable(Self::read_chunk_03043054),
            ),
            (0x03043055, ReadChunk::Skip),
            (
                0x03043056,
                ReadChunk::ReadSkippable(Self::read_chunk_03043056),
            ),
            (0x03043057, ReadChunk::Skip),
            (0x03043058, ReadChunk::Skip),
            (0x03043059, ReadChunk::Skip),
            (0x0304305A, ReadChunk::Skip),
            (0x0304305B, ReadChunk::Skip),
            (0x0304305C, ReadChunk::Skip),
            (0x0304305D, ReadChunk::Skip),
            (0x0304305E, ReadChunk::Skip),
            (
                0x0304305F,
                ReadChunk::ReadSkippable(Self::read_chunk_0304305f),
            ),
            (0x03043060, ReadChunk::Skip),
            (0x03043061, ReadChunk::Skip),
            (
                0x03043062,
                ReadChunk::ReadSkippable(Self::read_chunk_03043062),
            ),
            (
                0x03043063,
                ReadChunk::ReadSkippable(Self::read_chunk_03043063),
            ),
            (0x03043064, ReadChunk::Skip),
            (0x03043065, ReadChunk::Skip),
            (0x03043067, ReadChunk::Skip),
            (
                0x03043068,
                ReadChunk::ReadSkippable(Self::read_chunk_03043068),
            ),
            (0x03043069, ReadChunk::Skip),
        ]
    }
}
