/// Media tracker types.
pub mod media;

use crate::error::{ReadResult, WriteResult};
use crate::gbx::{Class, ReadBody, ReadChunk, ReadChunkFn, ReadHeader, WriteBody, WriteHeader};
use crate::reader::{self, Reader};
use crate::types::{RcStr, Vec3};
use crate::writer::{self, Writer};
use crate::{gbx, FileRef, Ghost};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use quick_xml::events::attributes::Attributes;
use quick_xml::events::Event;
use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, Write};
use std::ops::Sub;
use std::path::Path;

/// Medal times of a map.
#[derive(Clone, Hash, Debug)]
pub struct MedalTimes {
    /// Bronze medal time in milliseconds.
    pub bronze: u32,
    /// Silver medal time in milliseconds.
    pub silver: u32,
    /// Gold medal time in milliseconds.
    pub gold: u32,
    /// Author medal time in milliseconds.
    pub author: u32,
}

/// Cardinal direction of a block.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive)]
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

impl PartialOrd for LightmapQuality {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LightmapQuality {
    fn cmp(&self, other: &Self) -> Ordering {
        let order: fn(LightmapQuality) -> u8 = |lightmap_quality| match lightmap_quality {
            Self::Lowest => 0,
            Self::VeryLow => 1,
            Self::Low => 2,
            Self::Normal => 3,
            Self::High => 4,
            Self::VeryHigh => 5,
            Self::Highest => 6,
        };

        order(*self).cmp(&order(*other))
    }
}

/// Animation phase offset of a moving item.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[derive(Clone, Default, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, TryFromPrimitive)]
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
#[derive(Default, Debug)]
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
    pub skin: Option<Box<Skin>>,
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
#[derive(Default, Debug)]
pub struct FreeBlock {
    /// Id of the block's model.
    pub model_id: RcStr,
    /// Skin of the block, e.g. for signs.
    pub skin: Option<Box<Skin>>,
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
#[derive(Debug)]
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
/// Change the validation status and medal times of a map.
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut map = gbx::Map::read_from_file("MyMap.Map.Gbx")?;
///
/// map.medal_times = Some(gbx::map::MedalTimes {
///     bronze: 400,
///     silver: 300,
///     gold: 200,
///     author: 100,
/// });
///
/// map.write_to_file("MyMap.Map.Gbx")?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct Map {
    /// Validation medal times of the map.
    pub medal_times: Option<MedalTimes>,
    /// Display cost of the map.
    pub cost: u32,
    /// Number of checkpoints needed to finish the map.
    pub num_cps: u32,
    /// Number of laps if the map is multilap.
    pub num_laps: Option<u32>,
    /// Unique id of the map.
    pub uid: RcStr,
    /// Unique id of the map author.
    pub author_uid: RcStr,
    /// Name of the map.
    pub name: String,
    /// `true` if the map has no stadium.
    pub no_stadium: bool,
    /// Optional thumbnail of the map as raw JPEG.
    pub thumbnail: Option<Vec<u8>>,
    /// Name of the map author.
    pub author_name: String,
    /// Zone of the map author.
    pub author_zone: String,
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
    /// All blocks inside the map, including grass blocks and clips.
    ///
    /// The `skin` and `waypoint_property` fields of the baked blocks are always `None`.
    pub baked_blocks: Vec<BlockType>,
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
    /// Ids of the files embedded in the map.
    ///
    /// The length is equal to the number of files in the `embedded_files` ZIP archive.
    pub embedded_file_ids: Vec<RcStr>,
    /// All files embedded in the map as a raw ZIP archive.
    pub embedded_files: Option<Vec<u8>>,
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

    /// Read a map from a file at the given `path`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> gbx::error::ReadResult<()> {
    /// let map = gbx::Map::read_from_file("MyMap.Map.Gbx")?;
    /// # Ok(())
    /// # }
    /// ```
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
        gbx::write(self, writer)?;

        Ok(())
    }

    /// Write the map to a file at the given `path`.
    ///
    /// This function will create the file if it does not exist, and will truncate it if it does.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> gbx::error::WriteResult {
    /// let map = gbx::Map::new();
    /// map.write_to_file("MyMap.Map.Gbx")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_file<P>(&self, path: P) -> WriteResult
    where
        P: AsRef<Path>,
    {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        self.write_to(writer)
    }
}

fn read_medal_times<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Option<MedalTimes>>
where
    R: Read,
{
    match (r.u32()?, r.u32()?, r.u32()?, r.u32()?) {
        (bronze, silver, gold, author)
            if bronze != 0xFFFFFFFF
                && silver != 0xFFFFFFFF
                && gold != 0xFFFFFFFF
                && author != 0xFFFFFFFF =>
        {
            Ok(Some(MedalTimes {
                bronze,
                silver,
                gold,
                author,
            }))
        }
        _ => Ok(None),
    }
}

impl Map {
    fn read_chunk_03043002<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        r.u8()?;
        r.u32()?;
        self.medal_times = read_medal_times(r)?;
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
}

fn does_deco_have_no_stadium(deco_id: &str) -> bool {
    match deco_id {
        "48x48Sunrise" => false,
        "48x48Day" => false,
        "48x48Sunset" => false,
        "48x48Night" => false,
        "NoStadium48x48Day" => true,
        "Day16x12" => true,
        _ => panic!(),
    }
}

impl Map {
    fn read_chunk_03043003<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
        I: BorrowMut<reader::IdState>,
    {
        r.u8()?;
        self.uid = r.id()?;
        r.u32()?;
        self.author_uid = r.id()?;
        self.name = r.string()?;
        r.u8()?;
        r.u32()?;
        r.u32()?;
        self.no_stadium = does_deco_have_no_stadium(r.id()?.as_str());
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
}

fn xml_attributes_to_map(attributes: Attributes) -> HashMap<String, String> {
    attributes
        .map(|attribute| {
            let attribute = attribute.unwrap();
            (
                String::from_utf8(attribute.key.local_name().as_ref().to_vec()).unwrap(),
                attribute.unescape_value().unwrap().into_owned(),
            )
        })
        .collect()
}

impl Map {
    fn read_chunk_03043005<R, I, N>(&mut self, r: &mut Reader<R, I, N>) -> ReadResult<()>
    where
        R: Read,
    {
        let xml = r.string()?;
        let mut xml_reader = quick_xml::Reader::from_str(&xml);

        match xml_reader.read_event().unwrap() {
            Event::Start(e) if e.local_name().as_ref() == b"header" => {
                let _attributes = xml_attributes_to_map(e.attributes());
            }
            _ => panic!(),
        }

        match xml_reader.read_event().unwrap() {
            Event::Empty(e) if e.local_name().as_ref() == b"ident" => {
                let attributes = xml_attributes_to_map(e.attributes());
                self.uid = RcStr::new(attributes.get("uid").unwrap().clone());
                self.name = attributes.get("name").unwrap().clone();
                self.author_uid = RcStr::new(attributes.get("author").unwrap().clone());
                self.author_zone = attributes.get("authorzone").unwrap().clone();
            }
            _ => panic!(),
        }

        match xml_reader.read_event().unwrap() {
            Event::Empty(e) if e.local_name().as_ref() == b"desc" => {
                let attributes = xml_attributes_to_map(e.attributes());
                self.cost = attributes.get("displaycost").unwrap().parse().unwrap();
            }
            _ => panic!(),
        }

        match xml_reader.read_event().unwrap() {
            Event::Empty(e) if e.local_name().as_ref() == b"playermodel" => {}
            _ => panic!(),
        }

        match xml_reader.read_event().unwrap() {
            Event::Empty(e) if e.local_name().as_ref() == b"times" => {
                let attributes = xml_attributes_to_map(e.attributes());

                self.medal_times = Some(MedalTimes {
                    bronze: attributes.get("bronze").unwrap().parse().unwrap(),
                    silver: attributes.get("silver").unwrap().parse().unwrap(),
                    gold: attributes.get("gold").unwrap().parse().unwrap(),
                    author: attributes.get("authortime").unwrap().parse().unwrap(),
                })
            }
            _ => panic!(),
        }

        match xml_reader.read_event().unwrap() {
            Event::Start(e) if e.local_name().as_ref() == b"deps" => {}
            _ => panic!(),
        }

        loop {
            match xml_reader.read_event().unwrap() {
                Event::Empty(e) if e.local_name().as_ref() == b"dep" => {
                    let _attributes = xml_attributes_to_map(e.attributes());
                }
                Event::End(e) if e.local_name().as_ref() == b"deps" => break,
                _ => panic!(),
            }
        }

        match xml_reader.read_event().unwrap() {
            Event::End(e) if e.local_name().as_ref() == b"header" => {}
            _ => panic!(),
        }

        match xml_reader.read_event().unwrap() {
            Event::Eof => {}
            _ => panic!(),
        }

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
            self.medal_times = read_medal_times(r)?;
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
            let _is_validated = r.bool()?;

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
        self.no_stadium = does_deco_have_no_stadium(r.id()?.as_str());
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
                r.optional_node_owned(0x03059000, Skin::read)?.map(Box::new)
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
        {
            let mut r = Reader::with_id_state(r.take(size as u64), reader::IdState::new());
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

    fn write_chunk_03043002<W, I, N>(&self, mut w: Writer<W, I, N>) -> WriteResult
    where
        W: Write,
    {
        w.u8(13)?;
        w.u32(0)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(318)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(1)?;
        w.u32(1)?;

        Ok(())
    }

    fn write_chunk_03043003<W, I, N>(&self, mut w: Writer<W, I, N>) -> WriteResult
    where
        W: Write,
        I: BorrowMut<writer::IdState>,
    {
        w.u8(13)?;
        w.id(None)?;
        w.u32(26)?;
        w.id(None)?;
        w.string(&self.name)?;
        w.u8(6)?;
        w.u32(0)?;
        w.u32(0)?;
        w.id(Some("48x48Day"))?;
        w.u32(26)?;
        w.id(Some("Nadeo"))?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.string("TrackMania\\TM_Race")?;
        w.u32(0)?;
        w.u64(0)?;
        w.u8(0)?;
        w.id(Some("TMStadium"))?;

        Ok(())
    }

    fn write_chunk_03043004<W, I, N>(&self, mut w: Writer<W, I, N>) -> WriteResult
    where
        W: Write,
    {
        w.u32(6)?;

        Ok(())
    }

    fn write_chunk_03043005<W, I, N>(&self, mut w: Writer<W, I, N>) -> WriteResult
    where
        W: Write,
    {
        w.u32(0)?;

        Ok(())
    }

    fn write_chunk_03043007<W, I, N>(&self, mut w: Writer<W, I, N>) -> WriteResult
    where
        W: Write,
    {
        match self.thumbnail {
            Some(ref thumbnail) => {
                w.u32(1)?;
                w.u32(thumbnail.len() as u32)?;
                w.bytes(b"<Thumbnail.jpg>")?;
                w.bytes(thumbnail)?;
                w.bytes(b"</Thumbnail.jpg>")?;
                w.bytes(b"<Comments>")?;
                w.string("")?;
                w.bytes(b"</Comments>")?;
            }
            None => w.u32(0)?,
        }

        Ok(())
    }

    fn write_chunk_03043008<W, I, N>(&self, mut w: Writer<W, I, N>) -> WriteResult
    where
        W: Write,
    {
        w.u32(1)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;

        Ok(())
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

impl<W, I, N> WriteHeader<W, I, N> for Map
where
    W: Write,
    I: BorrowMut<writer::IdState>,
{
    fn write_header_chunks<'a>() -> &'a [(u32, fn(&Self, Writer<W, I, N>) -> WriteResult)] {
        &[
            (0x03043002, Self::write_chunk_03043002),
            (0x03043003, Self::write_chunk_03043003),
            (0x03043004, Self::write_chunk_03043004),
            (0x03043005, Self::write_chunk_03043005),
            (0x03043007, Self::write_chunk_03043007),
            (0x03043008, Self::write_chunk_03043008),
        ]
    }
}

impl<W, I, N> WriteBody<W, I, N> for Map
where
    W: Write,
    I: BorrowMut<writer::IdState>,
    N: BorrowMut<writer::NodeState>,
{
    fn write_body(&self, w: &mut Writer<W, I, N>) -> WriteResult {
        w.u32(0x0304300D)?;
        w.id(None)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;

        w.u32(0x03043011)?;
        w.node(0x0301B000, |w| {
            w.u32(0x0301B000)?;
            w.u32(0)?;

            Ok(())
        })?;
        w.node(0x0305B000, |w| {
            w.u32(0x0305B001)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            w.u32(0x0305B004)?;
            w.u32(0xFFFFFFFF)?;
            w.u32(0xFFFFFFFF)?;
            w.u32(0xFFFFFFFF)?;
            w.u32(0xFFFFFFFF)?;
            w.u32(0)?;

            w.u32(0x0305B008)?;
            w.u32(60000)?;
            w.u32(0)?;

            w.skippable_chunk(0x0305B00A, |mut w| {
                w.u32(0)?;
                w.u32(0xFFFFFFFF)?;
                w.u32(0xFFFFFFFF)?;
                w.u32(0xFFFFFFFF)?;
                w.u32(0xFFFFFFFF)?;
                w.u32(60000)?;
                w.u32(0)?;

                Ok(())
            })?;

            w.u32(0x0305B00D)?;
            w.u32(0xFFFFFFFF)?;

            w.skippable_chunk(0x0305B00E, |mut w| {
                w.string("TrackMania\\TM_Race")?;
                w.u32(0)?;
                w.u32(0)?;

                Ok(())
            })?;

            Ok(())
        })?;
        w.u32(6)?;

        w.skippable_chunk(0x03043018, |mut w| {
            w.u32(0)?;
            w.u32(3)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043019, |mut w| {
            w.file_ref(None)?;

            Ok(())
        })?;

        w.u32(0x0304301F)?;
        w.id(None)?;
        w.u32(26)?;
        w.id(None)?;
        w.string(&self.name)?;
        w.id(Some("48x48Day"))?;
        w.u32(26)?;
        w.id(Some("Nadeo"))?;
        w.u32(48)?;
        w.u32(40)?;
        w.u32(48)?;
        w.u32(0)?;
        w.u32(6)?;
        w.u32(0)?;

        w.u32(0x03043022)?;
        w.u32(1)?;

        w.u32(0x03043024)?;
        w.file_ref(None)?;

        w.u32(0x03043025)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;
        w.u32(0)?;

        w.u32(0x03043026)?;
        w.u32(0xFFFFFFFF)?;

        w.u32(0x03043028)?;
        w.u32(0)?;
        w.u32(0)?;

        w.skippable_chunk(0x03043029, |mut w| {
            w.bytes(&[0; 16])?;
            w.u32(0x51F6B4C7)?;

            Ok(())
        })?;

        w.u32(0x0304302A)?;
        w.u32(0)?;

        w.skippable_chunk(0x03043034, |mut w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043036, |mut w| {
            w.f32(640.0)?;
            w.f32(181.01933)?;
            w.f32(640.0)?;
            w.f32(std::f32::consts::FRAC_PI_4)?;
            w.f32(std::f32::consts::FRAC_PI_4)?;
            w.f32(0.0)?;
            w.f32(90.0)?;
            w.f32(10.0)?;
            w.f32(0.0)?;
            w.f32(-1.0)?;
            w.f32(-1.0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043038, |mut w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304303E, |mut w| {
            w.u32(0)?;
            w.u32(10)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043040, |mut w| {
            let mut bytes = vec![];
            {
                let mut w = Writer::new(&mut bytes);
                w.u32(10)?;
                w.u32(0)?;
                w.u32(0)?;
                w.u32(0)?;
                w.u32(0)?;
                w.u32(0)?;
                w.u32(0)?;
                w.u32(0)?;
            }

            w.u32(7)?;
            w.u32(0)?;
            w.u32(bytes.len() as u32)?;
            w.bytes(&bytes)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043042, |mut w| {
            w.u32(1)?;
            w.u32(0)?;
            w.string("")?;
            w.string("")?;
            w.string("")?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043043, |mut w| {
            let mut bytes = vec![];
            {
                let mut w = Writer::new(&mut bytes);
                w.u32(0)?;
            }

            w.u32(0)?;
            w.u32(bytes.len() as u32)?;
            w.bytes(&bytes)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043044, |mut w| {
            let mut bytes = vec![];
            {
                let mut w = Writer::new(&mut bytes);

                w.u32(0x11002000)?;
                w.u32(6)?;
                w.u8(0)?;
                w.u8(0)?;
            }

            w.u32(0)?;
            w.u32(bytes.len() as u32)?;
            w.bytes(&bytes)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043048, |mut w| {
            w.u32(0)?;
            w.u32(6)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.u32(0x03043049)?;
        w.u32(2)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(0xFFFFFFFF)?;
        w.u32(3)?;
        w.u32(1)?;
        w.u32(3)?;

        w.skippable_chunk(0x0304304B, |mut w| {
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304304F, |mut w| {
            w.u32(3)?;
            w.u8(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043050, |mut w| {
            w.u32(3)?;
            w.u32(1)?;
            w.u32(3)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043051, |mut w| {
            w.u32(0)?;
            w.id(Some("TMStadium"))?;
            w.string("date=2023-01-13_16_25 git=116238-efed8bf632f GameVersion=3.3.0")?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043052, |mut w| {
            w.u32(0)?;
            w.u32(8)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043053, |mut w| {
            w.u32(3)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043054, |mut w| {
            let mut bytes = vec![];
            {
                let mut w = Writer::new(&mut bytes);
                w.u32(0)?;
                w.u32(0)?;
                w.u32(0)?;
            }

            w.u32(1)?;
            w.u32(0)?;
            w.u32(bytes.len() as u32)?;
            w.bytes(&bytes)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043056, |mut w| {
            w.u32(3)?;
            w.u32(0)?;
            w.u32(0xFFFFFFFF)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(300000)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043057, |mut w| {
            w.u32(5)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043059, |mut w| {
            w.u32(3)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.f32(20.0)?;
            w.f32(3.0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304305A, |mut w| {
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304305B, |mut w| {
            w.u32(0)?;
            w.u32(1)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(8)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304305C, |mut w| {
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304305D, |mut w| {
            w.u32(1)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304305E, |mut w| {
            w.u32(1)?;
            w.u32(0)?;
            w.u32(8)?;
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x0304305F, |mut w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043060, |mut w| {
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043061, |mut w| {
            w.u32(1)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043062, |mut w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043063, |mut w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043064, |mut w| {
            w.u32(0)?;
            w.u32(0)?;
            w.u32(4)?;
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043065, |mut w| {
            w.u32(0)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043067, |mut w| {
            w.u32(0)?;
            w.u32(0)?;
            w.u32(4)?;
            w.u32(0xFFFFFFFF)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043068, |mut w| {
            w.u32(1)?;

            Ok(())
        })?;

        w.skippable_chunk(0x03043069, |mut w| {
            w.u32(0)?;
            w.u32(0)?;

            Ok(())
        })?;

        Ok(())
    }
}
