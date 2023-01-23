/// Media block types.
pub mod block;

use crate::error::ReadResult;
use crate::reader::{self, Reader};
use crate::Vec3;
use std::borrow::BorrowMut;
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
    DepthOfField(block::DepthOfField),
    ToneMapping(block::ToneMapping),
    BloomHdr(block::BloomHdr),
    TimeSpeed(block::TimeSpeed),
    Manialink(block::Manialink),
    VehicleLight(block::VehicleLight),
    EditingCut(block::EditingCut),
    DirtyLens(block::DirtyLens),
    ColorGrading(block::ColorGrading),
    ManialinkUI(block::ManialinkUI),
    Fog(block::Fog),
    Entity(block::Entity),
    OpponentVisibility(block::OpponentVisibility),
}

#[derive(Clone)]
pub struct TrackSegment {
    pub start: f32,
    pub end: f32,
}

/// A media track.
#[derive(Clone)]
pub struct Track {
    /// All blocks of this track.
    pub blocks: Vec<Block>,
    /// Whether the last block of the track should remain active after the end time.
    pub keep_last_block_active: bool,
    /// Track segment which should be repeated after the end time.
    pub repeat_track_segment: Option<TrackSegment>,
}

/// A media clip.
#[derive(Clone, Default)]
pub struct Clip {
    /// All tracks of this clip.
    pub tracks: Vec<Track>,
    /// Name of the clip.
    pub name: String,
    /// Stop the clip when the player leaves the trigger.
    pub stop_on_leave: bool,
    /// Stop the clip when the player respawns.
    pub stop_on_respawn: bool,
    /// `true` if the clip can trigger before the start the a race.
    pub can_trigger_before_start: bool,
}

impl Clip {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let mut clip = Self::default();

        r.chunk_id(0x0307900D)?;
        r.u32()?; // 0
        r.u32()?; // 10
        clip.tracks = r.list(|r| {
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
                            0x03126000 => Block::DepthOfField(block::DepthOfField::read(r)?),
                            0x03127000 => Block::ToneMapping(block::ToneMapping::read(r)?),
                            0x03128000 => Block::BloomHdr(block::BloomHdr::read(r)?),
                            0x03129000 => Block::TimeSpeed(block::TimeSpeed::read(r)?),
                            0x0312A000 => Block::Manialink(block::Manialink::read(r)?),
                            0x03133000 => Block::VehicleLight(block::VehicleLight::read(r)?),
                            0x03145000 => Block::EditingCut(block::EditingCut::read(r)?),
                            0x03165000 => Block::DirtyLens(block::DirtyLens::read(r)?),
                            0x03186000 => Block::ColorGrading(block::ColorGrading::read(r)?),
                            0x03195000 => Block::ManialinkUI(block::ManialinkUI::read(r)?),
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
                let keep_last_block_active = r.bool()?;
                r.u32()?;
                let repeat_track_segment = r.bool()?;
                let start = r.f32()?;
                let end = r.f32()?;

                r.node_end()?;

                Ok(Track {
                    blocks,
                    keep_last_block_active,
                    repeat_track_segment: repeat_track_segment
                        .then_some(TrackSegment { start, end }),
                })
            })
        })?;
        clip.name = r.string()?;
        clip.stop_on_leave = r.bool()?;
        r.u32()?;
        clip.stop_on_respawn = r.bool()?;
        r.u32()?;
        r.f32()?;
        r.u32()?;

        r.optional_skippable_chunk(0x0307900E, |r| {
            r.u32()?;
            clip.can_trigger_before_start = r.bool()?;

            Ok(())
        })?;

        r.node_end()?;

        Ok(clip)
    }
}

/// Condition to trigger a media clip.
#[derive(Clone, Default, Debug)]
#[non_exhaustive]
pub enum Condition {
    #[default]
    None,
    RaceTimeLessThan {
        /// Race time in seconds >= 0.0.
        time: f32,
    },
    RaceTimeGreaterThan {
        /// Race time in seconds >= 0.0.
        time: f32,
    },
    /// Will only trigger if the clip with `Some(clip_index)` already triggered. Will never trigger if `None`.
    AlreadyTriggered { clip_index: Option<u32> },
    SpeedLessThan {
        /// Speed of the car >= 0.0.
        speed: f32,
    },
    SpeedGreaterThan {
        /// Speed of the car >= 0.0.
        speed: f32,
    },
    /// Will only trigger if the clip with `Some(clip_index)` has not already triggered. Will always trigger if `None`.
    NotAlreadyTriggered { clip_index: Option<u32> },
    /// Will only trigger `Some(count)` times. Will always trigger if `None`.
    MaxPlayCount { count: Option<u32> },
    RandomOnce {
        /// Propability of triggering between 0.0 and 1.0.
        probability: f32,
    },
    Random {
        /// Propability of triggering between 0.0 and 1.0.
        probablity: f32,
    },
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
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        r.chunk_id(0x0307A003)?;
        r.u32()?; // 10
        let clips = r.list(|r| r.node_owned(0x03079000, Clip::read))?;
        let triggers = r.list(|r| {
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            let condition = match r.u32()? {
                0 => {
                    r.f32()?;
                    Condition::None
                }
                1 => Condition::RaceTimeLessThan {
                    time: r.f32()?.max(0.0),
                },
                2 => Condition::RaceTimeGreaterThan {
                    time: r.f32()?.max(0.0),
                },
                3 => {
                    let clip_index = r.f32()?.trunc() as i32;

                    Condition::AlreadyTriggered {
                        clip_index: (clip_index >= 0 && clip_index < clips.len() as i32)
                            .then_some(clip_index as u32),
                    }
                }
                4 => Condition::SpeedLessThan {
                    speed: r.f32()?.max(0.0),
                },
                5 => Condition::SpeedGreaterThan {
                    speed: r.f32()?.max(0.0),
                },
                6 => {
                    let clip_index = r.f32()?.trunc() as i32;

                    Condition::NotAlreadyTriggered {
                        clip_index: (clip_index >= 0 && clip_index < clips.len() as i32)
                            .then_some(clip_index as u32),
                    }
                }
                7 => {
                    let count = r.f32()?.trunc() as i32;

                    Condition::MaxPlayCount {
                        count: (count >= 0).then_some(count as u32),
                    }
                }
                8 => Condition::RandomOnce {
                    probability: r.f32()?.clamp(0.0, 1.0),
                },
                9 => Condition::Random {
                    probablity: r.f32()?.clamp(0.0, 1.0),
                },
                _ => panic!(),
            };
            let coords = r.list(|r| r.vec3u32())?;

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
