/// Media block types.
pub mod block;

use crate::error::ReadResult;
use crate::reader::{IdState, NodeState, Reader};
use crate::Vec3;
use std::io::{Read, Seek};

/// A media block.
#[derive(Clone)]
#[non_exhaustive]
pub enum Block {
    Triangles2D(block::Triangles),
    Triangles3D(block::Triangles),
    FxColors(block::FxColors),
    FxBlurMotion(block::FxBlurMotion),
    CameraGame(block::CameraGame),
    Time(block::Time),
    CameraOrbital(block::CameraOrbital),
    CameraPath(block::CameraPath),
    CameraCustom(block::CameraCustom),
    CameraShakeEffect(block::CameraShakeEffect),
    Image(block::Image),
    MusicEffect(block::MusicEffect),
    Sound(block::Sound),
    Text(block::Text),
    Trails(block::Trails),
    TransitionFade(block::TransitionFade),
    DOF(block::DOF),
    ToneMapping(block::ToneMapping),
    BloomHdr(block::BloomHdr),
    TimeSpeed(block::TimeSpeed),
    Manialink(block::Manialink),
    VehicleLight(block::VehicleLight),
    Shoot(block::Shoot),
    DirtyLens(block::DirtyLens),
    ColorGrading(block::ColorGrading),
    Interface(block::Interface),
    Fog(block::Fog),
    Entity(block::Entity),
    OpponentVisibility(block::OpponentVisibility),
}

/// A media track.
#[derive(Clone)]
pub struct Track {
    /// All blocks of this track.
    pub blocks: Vec<Block>,
}

/// A media clip.
#[derive(Clone)]
pub struct Clip {
    /// All tracks of this clip.
    pub tracks: Vec<Track>,
}

impl Clip {
    pub fn read<R>(r: &mut Reader<R, IdState, NodeState>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x0307900D)?;
        r.u32()?; // 0
        r.u32()?; // 10
        let tracks = r.list(|r| {
            r.node_owned(0x03078000, |r| {
                r.chunk_id(0x03078001)?;
                let _name = r.string()?;
                r.u32()?; // 10

                let blocks = r.list(|r| {
                    r.any_node_owned(|r, class_id| {
                        let block = match class_id {
                            0x0304B000 => Block::Triangles3D(block::Triangles::read(r)?),
                            0x0304C000 => Block::Triangles3D(block::Triangles::read(r)?),
                            0x03080000 => Block::FxColors(block::FxColors::read(r)?),
                            0x03082000 => Block::FxBlurMotion(block::FxBlurMotion::read(r)?),
                            0x03084000 => Block::CameraGame(block::CameraGame::read(r)?),
                            0x03085000 => Block::Time(block::Time::read(r)?),
                            0x030A0000 => Block::CameraOrbital(block::CameraOrbital::read(r)?),
                            0x030A1000 => Block::CameraPath(block::CameraPath::read(r)?),
                            0x030A2000 => Block::CameraCustom(block::CameraCustom::read(r)?),
                            0x030A4000 => {
                                Block::CameraShakeEffect(block::CameraShakeEffect::read(r)?)
                            }
                            0x030A5000 => Block::Image(block::Image::read(r)?),
                            0x030A6000 => Block::MusicEffect(block::MusicEffect::read(r)?),
                            0x030A7000 => Block::Sound(block::Sound::read(r)?),
                            0x030A8000 => Block::Text(block::Text::read(r)?),
                            0x030A9000 => Block::Trails(block::Trails::read(r)?),
                            0x030AB000 => Block::TransitionFade(block::TransitionFade::read(r)?),
                            0x03126000 => Block::DOF(block::DOF::read(r)?),
                            0x03127000 => Block::ToneMapping(block::ToneMapping::read(r)?),
                            0x03128000 => Block::BloomHdr(block::BloomHdr::read(r)?),
                            0x03129000 => Block::TimeSpeed(block::TimeSpeed::read(r)?),
                            0x0312A000 => Block::Manialink(block::Manialink::read(r)?),
                            0x03133000 => Block::VehicleLight(block::VehicleLight::read(r)?),
                            0x03145000 => Block::Shoot(block::Shoot::read(r)?),
                            0x03165000 => Block::DirtyLens(block::DirtyLens::read(r)?),
                            0x03186000 => Block::ColorGrading(block::ColorGrading::read(r)?),
                            0x03195000 => Block::Interface(block::Interface::read(r)?),
                            0x03199000 => Block::Fog(block::Fog::read(r)?),
                            0x0329F000 => Block::Entity(block::Entity::read(r)?),
                            0x0338B000 => {
                                Block::OpponentVisibility(block::OpponentVisibility::read(r)?)
                            }
                            _ => panic!("{class_id:08X}"),
                        };

                        r.node_end()?;

                        Ok(block)
                    })
                })?;
                r.u32()?; // 0xFFFFFFFF

                r.chunk_id(0x03078005)?;
                r.u32()?;
                r.u32()?;
                r.u32()?;
                r.u32()?;
                r.u32()?;
                r.u32()?;

                r.node_end()?;

                Ok(Track { blocks })
            })
        })?;
        let _name = r.string()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        r.skip_optional_chunk(0x0307900E)?;

        r.node_end()?;

        Ok(Clip { tracks })
    }
}

/// Condition to activate a media clip.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Condition {
    None,
    RaceTimeLessThan { time: f32 },
    RaceTimeGreaterThan { time: f32 },
    AlreadyTriggered { clip_index: u32 },
    SpeedLessThan { speed: f32 },
    SpeedGreaterThan { speed: f32 },
    NotAlreadyTriggered { clip_index: u32 },
    MaxPlayCount { count: u32 },
    RandomOnce { probability: f32 },
    Random { probablity: f32 },
}

/// A media clip and its trigger conditions.
#[derive(Clone)]
pub struct ClipTrigger {
    /// The clip which gets activated by the trigger conditions.
    pub clip: Clip,
    /// Condition which needs to be met to trigger the clip.
    pub condition: Condition,
    /// Coords at which the clip gets triggered.
    pub coords: Vec<Vec3<u32>>,
}

/// A media clip group.
#[derive(Clone)]
pub struct ClipGroup {
    /// All the clips and associated triggers in this clip group.
    pub clips: Vec<ClipTrigger>,
}

impl ClipGroup {
    pub fn read<R>(r: &mut Reader<R, IdState, NodeState>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x0307A003)?;
        r.u32()?; // 10
        let clips = r.list(|r| r.node_owned(0x03079000, Clip::read))?;
        let triggers = r.list(|r| {
            r.skip(16)?;
            let condition = match r.u32()? {
                0 => {
                    r.skip(4)?;
                    Condition::None
                }
                1 => Condition::RaceTimeLessThan { time: r.f32()? },
                2 => Condition::RaceTimeGreaterThan { time: r.f32()? },
                3 => Condition::AlreadyTriggered {
                    clip_index: r.f32()? as u32,
                },
                4 => Condition::SpeedLessThan { speed: r.f32()? },
                5 => Condition::SpeedGreaterThan { speed: r.f32()? },
                6 => Condition::NotAlreadyTriggered {
                    clip_index: r.f32()? as u32,
                },
                7 => Condition::MaxPlayCount {
                    count: r.f32()? as u32,
                },
                8 => Condition::RandomOnce {
                    probability: r.f32()?,
                },
                9 => Condition::Random {
                    probablity: r.f32()?,
                },
                _ => panic!(),
            };
            let coords = r.list(|r| {
                let x = r.u32()?;
                let y = r.u32()?;
                let z = r.u32()?;

                Ok(Vec3 { x, y, z })
            })?;

            Ok((condition, coords))
        })?;

        let clips = clips
            .into_iter()
            .zip(triggers)
            .map(|(clip, (condition, coords))| ClipTrigger {
                clip,
                condition,
                coords,
            })
            .collect();

        r.node_end()?;

        Ok(Self { clips })
    }
}
