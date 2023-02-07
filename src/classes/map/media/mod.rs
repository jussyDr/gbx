/// Media block types.
pub mod block;

use crate::read;
use crate::reader::{self, Reader};
use crate::Vec3;
use std::borrow::BorrowMut;
use std::io::{Read, Seek};

/// A media block.
#[derive(Clone)]
#[non_exhaustive]
pub enum Block {
    /// 2D triangles media block.
    Triangles2D(block::Triangles),
    /// 3D triangles media block.
    Triangles3D(block::Triangles),
    /// Color media block.
    Color(block::Color),
    /// Motion blur media block.
    MotionBlur(block::MotionBlur),
    /// Player camera media block.
    PlayerCamera(block::PlayerCamera),
    /// Time media block.
    Time(block::Time),
    /// Orbital camera media block.
    OrbitalCamera(block::OrbitalCamera),
    /// Path camera media block.
    PathCamera(block::PathCamera),
    /// Custom camera media block.
    CustomCamera(block::CustomCamera),
    /// Camera shake effect media block.
    CameraShakeEffect(block::CameraShakeEffect),
    /// Image media block.
    Image(block::Image),
    /// Music volume media block.
    MusicVolume(block::MusicVolume),
    /// Sound media block.
    Sound(block::Sound),
    /// Text media block.
    Text(block::Text),
    /// Trails media block.
    Trails(block::Trails),
    /// Transition fade media block.
    TransitionFade(block::TransitionFade),
    /// Depth of field media block.
    DepthOfField(block::DepthOfField),
    /// Tone mapping media block.
    ToneMapping(block::ToneMapping),
    /// Bloom media block.
    Bloom(block::Bloom),
    /// Time speed media block.
    TimeSpeed(block::TimeSpeed),
    /// Manialink media block.
    Manialink(block::Manialink),
    /// Vehicle light media block.
    VehicleLight(block::VehicleLight),
    /// Editing cut media block.
    EditingCut(block::EditingCut),
    /// Dirty lens media block.
    DirtyLens(block::DirtyLens),
    /// Color grading media block.
    ColorGrading(block::ColorGrading),
    /// Manialink interface media block.
    ManialinkInterface(block::ManialinkInterface),
    /// Fog media block.
    Fog(block::Fog),
    /// Entity media block.
    Entity(block::Entity),
    /// Opponent visibility media block.
    OpponentVisibility(block::OpponentVisibility),
}

/// Segment of a media track.
#[derive(Clone)]
pub struct TrackSegment {
    /// Start time of the segment. [0, ∞)
    pub start_time: f32,
    /// End time of the segment. [0, ∞)
    pub end_time: f32,
}

/// A media track.
#[derive(Clone)]
pub struct Track {
    /// All blocks of the track.
    pub blocks: Vec<Block>,
    /// `true` if the last block of the track should remain active after its end time.
    pub keep_last_block_active: bool,
    /// Track segment which should be repeated after the last block.
    pub repeat_track_segment: Option<TrackSegment>,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            blocks: Vec::default(),
            keep_last_block_active: true,
            repeat_track_segment: Option::default(),
        }
    }
}

/// A media clip.
#[derive(Clone)]
pub struct Clip {
    /// All tracks of the clip.
    pub tracks: Vec<Track>,
    /// Name of the clip.
    pub name: String,
    /// Stop the clip when the player leaves the trigger coords.
    pub stop_on_leave: bool,
    /// Stop the clip when the player respawns.
    pub stop_on_respawn: bool,
    /// `true` if the clip can trigger before the start of a race.
    pub can_trigger_before_start: bool,
}

impl Clip {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> read::Result<Self>
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
                            0x03080000 => Block::Color(block::Color::read(r)?),
                            0x03082000 => Block::MotionBlur(block::MotionBlur::read(r)?),
                            0x03084000 => Block::PlayerCamera(block::PlayerCamera::read(r)?),
                            0x03085000 => Block::Time(block::Time::read(r)?),
                            0x030A0000 => Block::OrbitalCamera(block::OrbitalCamera::read(r)?),
                            0x030A1000 => Block::PathCamera(block::PathCamera::read(r)?),
                            0x030A2000 => Block::CustomCamera(block::CustomCamera::read(r)?),
                            0x030A4000 => {
                                Block::CameraShakeEffect(block::CameraShakeEffect::read(r)?)
                            }
                            0x030A5000 => Block::Image(block::Image::read(r)?),
                            0x030A6000 => Block::MusicVolume(block::MusicVolume::read(r)?),
                            0x030A7000 => Block::Sound(block::Sound::read(r)?),
                            0x030A8000 => Block::Text(block::Text::read(r)?),
                            0x030A9000 => Block::Trails(block::Trails::read(r)?),
                            0x030AB000 => Block::TransitionFade(block::TransitionFade::read(r)?),
                            0x03126000 => Block::DepthOfField(block::DepthOfField::read(r)?),
                            0x03127000 => Block::ToneMapping(block::ToneMapping::read(r)?),
                            0x03128000 => Block::Bloom(block::Bloom::read(r)?),
                            0x03129000 => Block::TimeSpeed(block::TimeSpeed::read(r)?),
                            0x0312A000 => Block::Manialink(block::Manialink::read(r)?),
                            0x03133000 => Block::VehicleLight(block::VehicleLight::read(r)?),
                            0x03145000 => Block::EditingCut(block::EditingCut::read(r)?),
                            0x03165000 => Block::DirtyLens(block::DirtyLens::read(r)?),
                            0x03186000 => Block::ColorGrading(block::ColorGrading::read(r)?),
                            0x03195000 => {
                                Block::ManialinkInterface(block::ManialinkInterface::read(r)?)
                            }
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
                let start_time = r.f32()?;
                let end_time = r.f32()?;

                r.node_end()?;

                Ok(Track {
                    blocks,
                    keep_last_block_active,
                    repeat_track_segment: repeat_track_segment.then_some(TrackSegment {
                        start_time,
                        end_time,
                    }),
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

impl Default for Clip {
    fn default() -> Self {
        Self {
            tracks: Vec::default(),
            name: String::default(),
            stop_on_leave: false,
            stop_on_respawn: true,
            can_trigger_before_start: false,
        }
    }
}

/// Condition to trigger a media clip.
#[derive(Clone, Default, Debug)]
#[non_exhaustive]
pub enum Condition {
    #[default]
    None,
    RaceTimeLessThan {
        /// Race time in seconds. [0.0, ∞)
        time: f32,
    },
    RaceTimeGreaterThan {
        /// Race time in seconds. [0.0, ∞)
        time: f32,
    },
    /// Will only trigger if the clip with `Some(clip_index)` already triggered. Will never trigger if `None`.
    AlreadyTriggered { clip_index: Option<u32> },
    SpeedLessThan {
        /// Speed of the car. [0.0, ∞)
        speed: f32,
    },
    SpeedGreaterThan {
        /// Speed of the car. [0.0, ∞)
        speed: f32,
    },
    /// Will only trigger if the clip with `Some(clip_index)` has not already triggered. Will always trigger if `None`.
    NotAlreadyTriggered { clip_index: Option<u32> },
    /// Will only trigger `Some(count)` times. Will always trigger if `None`.
    MaxPlayCount { count: Option<u32> },
    RandomOnce {
        /// Probability of triggering. [0.0, 1.0]
        probability: f32,
    },
    Random {
        /// Probability of triggering. [0.0, 1.0]
        probablity: f32,
    },
}

/// A media clip and its trigger conditions.
#[derive(Clone, Default)]
pub struct ClipTrigger {
    /// The clip which gets activated by the trigger conditions.
    pub clip: Clip,
    /// Condition which needs to be met to trigger the clip.
    pub condition: Condition,
    /// Coords at which the clip gets triggered.
    pub coords: Vec<Vec3<u32>>,
}

/// A media clip group.
#[derive(Clone, Default)]
pub struct ClipGroup {
    /// All the clips and associated triggers in this clip group.
    pub clips: Vec<ClipTrigger>,
}

impl ClipGroup {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> read::Result<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        let mut clip_group = Self::default();

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

        clip_group.clips = clips
            .into_iter()
            .zip(triggers)
            .map(|(clip, (condition, coords))| ClipTrigger {
                clip,
                condition,
                coords,
            })
            .collect();

        r.node_end()?;

        Ok(clip_group)
    }
}
