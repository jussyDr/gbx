/// Media block key types.
pub mod key;

use crate::error::ReadResult;
use crate::ghost::EntityRecordData;
use crate::reader::{IdState, NodeState, Reader};
use crate::{FileRef, Vec3};
use std::io::{Read, Seek};

/// Generic media block keys.
#[derive(Clone)]
pub struct EffectSimi;

impl EffectSimi {
    fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x07010005)?;
        let _keys = r.list(|r| {
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
        })?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        r.node_end()?;

        Ok(Self)
    }
}

/// Triangles 2D or 3D media block.
#[derive(Clone)]
pub struct Triangles;

impl Triangles {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x03029001)?;
        let _keys = r.list(|r| {
            r.u32()?;

            Ok(())
        })?;
        let num_keys = r.u32()?;
        let num_vertices = r.u32()?;
        r.repeat(num_keys as usize, |r| {
            r.repeat(num_vertices as usize, |r| {
                r.u32()?;
                r.u32()?;
                r.u32()?;

                Ok(())
            })?;

            Ok(())
        })?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        r.skip_optional_chunk(0x03029002)?;

        Ok(Self)
    }
}

/// Fx colors media block.
#[derive(Clone)]
pub struct FxColors {
    pub keys: Vec<key::FxColors>,
}

impl FxColors {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03080003)?;
        let keys = r.list(|r| {
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

            Ok(key::FxColors)
        })?;

        Ok(Self { keys })
    }
}

/// Fx blur motion block.
#[derive(Clone)]
pub struct FxBlurMotion;

impl FxBlurMotion {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03082000)?;
        r.u32()?;
        r.u32()?;

        Ok(Self)
    }
}

/// Camera game media block.
#[derive(Clone)]
pub struct CameraGame;

impl CameraGame {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03084007)?;
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

        Ok(Self)
    }
}

/// Time media block.
#[derive(Clone)]
pub struct Time {
    pub keys: Vec<key::Time>,
}

impl Time {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03085000)?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let time_value = r.f32()?;
            let tangent = r.f32()?;

            Ok(key::Time {
                time,
                time_value,
                tangent,
            })
        })?;

        Ok(Self { keys })
    }
}

/// Camera orbital media block
#[derive(Clone)]
pub struct CameraOrbital;

impl CameraOrbital {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A0001)?;
        r.u32()?;
        r.list(|r| {
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
            r.u8()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Camera path media block.
#[derive(Clone)]
pub struct CameraPath;

impl CameraPath {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A1003)?;
        r.u32()?; // 5
        let _keys = r.list(|r| {
            r.u32()?; // 0
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
        })?;

        Ok(Self)
    }
}

/// Camera custom media block.
#[derive(Clone)]
pub struct CameraCustom;

impl CameraCustom {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A2006)?;
        r.u32()?;
        let _keys = r.list(|r| {
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
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Camera shake effect media block.
#[derive(Clone)]
pub struct CameraShakeEffect {
    pub keys: Vec<key::CameraShakeEffect>,
}

impl CameraShakeEffect {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x030A4000)?;
        let keys = r.list(|r| {
            r.skip(4)?;
            let intensity = r.f32()?;
            let speed = r.f32()?;

            Ok(key::CameraShakeEffect { intensity, speed })
        })?;

        Ok(Self { keys })
    }
}

/// Image media block.
#[derive(Clone)]
pub struct Image {
    pub effect: EffectSimi,
    pub image: Option<FileRef>,
}

impl Image {
    pub(crate) fn read<R, I>(r: &mut Reader<R, I, NodeState>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A5000)?;
        let effect = r.node_owned(0x07010000, EffectSimi::read)?;
        let image = r.optional_file_ref()?;

        Ok(Self { effect, image })
    }
}

/// Music effect media block.
#[derive(Clone)]
pub struct MusicEffect {
    pub keys: Vec<key::MusicEffect>,
}

impl MusicEffect {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x030A6001)?;
        let keys = r.list(|r| {
            r.skip(4)?;
            let music_volume = r.f32()?;
            let sound_volume = r.f32()?;

            Ok(key::MusicEffect {
                music_volume,
                sound_volume,
            })
        })?;

        Ok(Self { keys })
    }
}

/// Sound media block.
#[derive(Clone)]
pub struct Sound {
    pub play_count: u32,
    pub is_looping: bool,
    pub is_music: bool,
    pub sound: Option<FileRef>,
    pub keys: Vec<key::Sound>,
}

impl Sound {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x030A7003)?;
        r.skip(4)?;
        let play_count = r.u32()?;
        let is_looping = r.bool()?;
        let is_music = r.bool()?;
        r.u32()?;
        let _audio_to_speech = r.bool()?;
        let _audio_to_speech_target = r.u32()?;

        r.chunk_id(0x030A7004)?;
        let sound = r.optional_file_ref()?;
        r.u32()?;
        let keys = r.list(|r| {
            r.u32()?;
            let volume = r.f32()?;
            r.u32()?;
            let x = r.f32()?;
            let y = r.f32()?;
            let z = r.f32()?;

            Ok(key::Sound {
                volume,
                position: Vec3 { x, y, z },
            })
        })?;

        Ok(Self {
            play_count,
            is_looping,
            is_music,
            sound,
            keys,
        })
    }
}

/// Text media block.
#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub effect: EffectSimi,
    pub color: Vec3<f32>,
}

impl Text {
    pub(crate) fn read<R, I>(r: &mut Reader<R, I, NodeState>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A8001)?;
        let text = r.string()?;
        let effect = r.node_owned(0x07010000, EffectSimi::read)?;

        r.chunk_id(0x030A8002)?;
        let red = r.f32()?;
        let green = r.f32()?;
        let blue = r.f32()?;

        Ok(Self {
            effect,
            text,
            color: Vec3 {
                x: red,
                y: green,
                z: blue,
            },
        })
    }
}

/// Trails media block.
#[derive(Clone)]
pub struct Trails {
    pub start_time: f32,
    pub end_time: f32,
}

impl Trails {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A9000)?;
        let start_time = r.f32()?;
        let end_time = r.f32()?;

        Ok(Self {
            start_time,
            end_time,
        })
    }
}

/// Transition fade media block.
#[derive(Clone)]
pub struct TransitionFade {
    pub keys: Vec<key::TransitionFade>,
    pub color: Vec3<f32>,
}

impl TransitionFade {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030AB000)?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let opacity = r.f32()?;

            Ok(key::TransitionFade { time, opacity })
        })?;
        let red = r.f32()?;
        let green = r.f32()?;
        let blue = r.f32()?;
        r.u32()?;

        Ok(Self {
            keys,
            color: Vec3 {
                x: red,
                y: green,
                z: blue,
            },
        })
    }
}

/// Depth-of-field media block.
#[derive(Clone)]
pub struct DOF;

impl DOF {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03126002)?;
        let _keys = r.list(|r| {
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Tone mapping media block
#[derive(Clone)]
pub struct ToneMapping;

impl ToneMapping {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03127004)?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Bloom high dynamic range media block.
#[derive(Clone)]
pub struct BloomHdr {
    pub keys: Vec<key::BloomHdr>,
}

impl BloomHdr {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x03128002)?;
        let keys = r.list(|r| {
            r.skip(4)?;
            let intensity = r.f32()?;
            let streaks_intensity = r.f32()?;
            let streaks_attenuation = r.f32()?;

            Ok(key::BloomHdr {
                intensity,
                streaks_intensity,
                streaks_attenuation,
            })
        })?;

        Ok(Self { keys })
    }
}

/// Time speed media block.
#[derive(Clone)]
pub struct TimeSpeed;

impl TimeSpeed {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03129000)?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Manialink media block.
#[derive(Clone)]
pub struct Manialink;

impl Manialink {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x0312A001)?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.string()?;

        Ok(Self)
    }
}

/// Vehicle light media block.
#[derive(Clone)]
pub struct VehicleLight;

impl VehicleLight {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03133000)?;
        r.u32()?;
        r.u32()?;

        r.chunk_id(0x03133001)?;
        r.u32()?;

        Ok(Self)
    }
}

/// Shoot media block.
#[derive(Clone)]
pub struct Shoot;

impl Shoot {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03145000)?;
        r.u32()?;
        r.u32()?;

        Ok(Self)
    }
}

/// Dirty lens media block.
#[derive(Clone)]
pub struct DirtyLens;

impl DirtyLens {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03165000)?;
        r.u32()?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Color grading media block.
#[derive(Clone)]
pub struct ColorGrading;

impl ColorGrading {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03186000)?;
        r.optional_file_ref()?;

        r.chunk_id(0x03186001)?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Interface media block.
#[derive(Clone)]
pub struct Interface;

impl Interface {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03195000)?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.string()?;

        Ok(Self)
    }
}

/// Fog media block.
#[derive(Clone)]
pub struct Fog;

impl Fog {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03199000)?;
        r.u32()?;
        let _keys = r.list(|r| {
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
        })?;

        Ok(Self)
    }
}

/// Entity media block.
#[derive(Clone)]
pub struct Entity;

impl Entity {
    pub(crate) fn read<R>(r: &mut Reader<R, IdState, NodeState>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x0329F000)?;
        let version = r.u32()?;
        r.node(0x0911F000, EntityRecordData::read)?;
        r.u32()?; // ?
        r.list(|r| {
            r.u32()?;

            Ok(())
        })?;
        r.u32()?; // 0
        r.u32()?; // 0
        r.u32()?; // 0
        r.u32()?; // 0
        r.optional_id()?; // "CarSport"
        r.u32()?;
        r.optional_id()?; // "Nadeo"
        r.u32()?; // f32
        r.u32()?; // f32
        r.u32()?; // f32
        r.list(|r| {
            r.optional_file_ref()?;

            Ok(())
        })?;
        r.u32()?;
        r.list(|r| {
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            if version >= 9 {
                r.u32()?;
            }

            Ok(())
        })?;
        if version >= 7 {
            r.string()?;
        }
        if version >= 8 {
            r.u32()?;
        }

        r.optional_chunk(0x0329F002, |r| {
            r.u32()?;

            Ok(())
        })?;

        Ok(Self)
    }
}

/// Opponent visibility media block.
#[derive(Clone)]
pub struct OpponentVisibility;

impl OpponentVisibility {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x0338B000)?;
        r.u32()?;
        r.u32()?;

        r.chunk_id(0x0338B001)?;
        r.u32()?;

        Ok(Self)
    }
}
