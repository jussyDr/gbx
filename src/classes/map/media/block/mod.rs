/// Media block key types.
pub mod key;

use crate::error::ReadResult;
use crate::ghost::EntityRecord;
use crate::reader::{self, Reader};
use crate::{FileRef, InternalFileRef, Rgb};
use int_enum::TryFromInteger;
use std::borrow::BorrowMut;
use std::io::{Read, Seek};

/// Effect of a media block.
#[derive(Clone)]
pub struct Effect {
    /// Keys of the effect.
    pub keys: Vec<key::Effect>,
}

impl Effect {
    fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x07010005)?;
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

            Ok(key::Effect)
        })?;
        r.u32()?;
        r.u32()?;
        r.u32()?;
        r.u32()?;

        r.node_end()?;

        Ok(Self { keys })
    }
}

/// 2D or 3D triangles media block.
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

/// Color media block.
#[derive(Clone)]
pub struct Color {
    /// Keys of the media block.
    pub keys: Vec<key::Color>,
}

impl Color {
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

            Ok(key::Color)
        })?;

        Ok(Self { keys })
    }
}

/// Motion blur media block.
#[derive(Clone)]
pub struct MotionBlur;

impl MotionBlur {
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

/// Player camera media block.
#[derive(Clone)]
pub struct PlayerCamera;

impl PlayerCamera {
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
    /// Keys of the media block.
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

/// Orbital camera media block
#[derive(Clone)]
pub struct OrbitalCamera;

impl OrbitalCamera {
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

/// Path camera media block.
#[derive(Clone)]
pub struct PathCamera;

impl PathCamera {
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

/// Custom camera media block.
#[derive(Clone)]
pub struct CustomCamera {
    /// Keys of the media block.
    pub keys: Vec<key::CustomCamera>,
}

impl CustomCamera {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x030A2006)?;
        r.u32()?;
        let keys = r.list(|r| {
            let _time = r.f32()?;
            let _interpolation = r.u32()?;
            let _anchor_rotation = r.bool()?;
            let _anchor = r.u32()?; // 0xFFFFFFFF = None, 0 = Local Player
            let _show_anchor = r.bool()?;
            let _target = r.u32()?; // 0xFFFFFFFF = None, 0 = Local Player
            let _x = r.f32()?;
            let _y = r.f32()?;
            let _z = r.f32()?;
            let _pitch = r.f32()?;
            let _yaw = r.f32()?;
            let _roll = r.f32()?;
            let _fov = r.f32()?;
            r.u32()?;
            r.u32()?;
            r.u32()?;
            let _z_near = r.f32()?;
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

            Ok(key::CustomCamera)
        })?;

        Ok(Self { keys })
    }
}

/// Camera shake effect media block.
#[derive(Clone)]
pub struct CameraShakeEffect {
    /// Keys of the media block.
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
    /// Effect of the image.
    pub effect: Effect,
    /// Optional reference to the image file.
    pub image: Option<FileRef>,
}

impl Image {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
        N: BorrowMut<reader::NodeState>,
    {
        r.chunk_id(0x030A5000)?;
        let effect = r.node_owned(0x07010000, Effect::read)?;
        let image = r.optional_file_ref()?;

        Ok(Self { effect, image })
    }
}

/// Music volume media block.
#[derive(Clone)]
pub struct MusicVolume {
    /// Keys of the media block.
    pub keys: Vec<key::MusicVolume>,
}

impl MusicVolume {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A6001)?;
        let keys = r.list(|r| {
            r.u32()?;
            let music_volume = r.f32()?;
            let sound_volume = r.f32()?;

            Ok(key::MusicVolume {
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
    /// Number of times to play the sound.
    pub play_count: u32,
    /// `true` if the sound should loop.
    pub is_looping: bool,
    /// `true` if the sound is music.
    pub is_music: bool,
    /// Optional sound.
    pub sound: Option<FileRef>,
    /// Keys of the media block.
    pub keys: Vec<key::Sound>,
}

impl Sound {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x030A7003)?;
        r.u32()?;
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
            let position = r.vec3f32()?;

            Ok(key::Sound { volume, position })
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
    /// The text.
    pub text: String,
    /// Effect of the text.
    pub effect: Effect,
    /// Color of the text.
    pub color: Rgb,
}

impl Text {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
        N: BorrowMut<reader::NodeState>,
    {
        r.chunk_id(0x030A8001)?;
        let text = r.string()?;
        let effect = r.node_owned(0x07010000, Effect::read)?;

        r.chunk_id(0x030A8002)?;
        let red = r.f32()?;
        let green = r.f32()?;
        let blue = r.f32()?;

        Ok(Self {
            effect,
            text,
            color: Rgb { red, green, blue },
        })
    }
}

/// Trails media block.
#[derive(Clone)]
pub struct Trails {
    /// Start time of the block in seconds. [0.0, ∞)
    pub start_time: f32,
    /// End time of the block in seconds. [0.0, ∞)
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
    /// Keys of the media block.
    pub keys: Vec<key::TransitionFade>,
    /// Color of the fade.
    pub color: Rgb,
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
            color: Rgb { red, green, blue },
        })
    }
}

/// Depth of field media block.
#[derive(Clone)]
pub struct DepthOfField {
    /// Keys of the media block.
    pub keys: Vec<key::DepthOfField>,
}

impl DepthOfField {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03126002)?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let focus_distance = r.f32()?;
            let lens_size = r.f32()?;
            let _target = r.u32()?;
            let target_position = r.vec3f32()?;

            Ok(key::DepthOfField {
                time,
                focus_distance,
                lens_size,
                target_position,
            })
        })?;

        Ok(Self { keys })
    }
}

/// Tone mapping media block
#[derive(Clone)]
pub struct ToneMapping {
    /// Keys of the media block.
    pub keys: Vec<key::ToneMapping>,
}

impl ToneMapping {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03127004)?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let exposure = r.f32()?;
            let max_hdr = r.f32()?;
            let light_trail_scale = r.f32()?;
            r.u32()?;

            Ok(key::ToneMapping {
                time,
                exposure,
                max_hdr,
                light_trail_scale,
            })
        })?;

        Ok(Self { keys })
    }
}

/// Bloom media block.
#[derive(Clone)]
pub struct Bloom {
    /// Keys of the media block.
    pub keys: Vec<key::Bloom>,
}

impl Bloom {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
    {
        r.chunk_id(0x03128002)?;
        let keys = r.list(|r| {
            r.u32()?;
            let intensity = r.f32()?;
            let streaks_intensity = r.f32()?;
            let streaks_attenuation = r.f32()?;

            Ok(key::Bloom {
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
pub struct TimeSpeed {
    /// Keys of the media block.
    pub keys: Vec<key::TimeSpeed>,
}

impl TimeSpeed {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03129000)?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let speed = r.f32()?;

            Ok(key::TimeSpeed { time, speed })
        })?;

        Ok(Self { keys })
    }
}

/// Manialink media block.
#[derive(Clone)]
pub struct Manialink {
    /// Start time of the block in seconds. [0.0, ∞)
    pub start_time: f32,
    /// End time of the block in seconds. [0.0, ∞)
    pub end_time: f32,
    /// URL to the manialink.
    pub url: String,
}

impl Manialink {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x0312A001)?;
        r.u32()?;
        let start_time = r.f32()?;
        let end_time = r.f32()?;
        let url = r.string()?;

        Ok(Self {
            start_time,
            end_time,
            url,
        })
    }
}

/// Vehicle light media block.
#[derive(Clone, Debug)]
pub struct VehicleLight {
    /// Start time of the block in seconds. [0.0, ∞)
    pub start_time: f32,
    /// End time of the block in seconds. [0.0, ∞)
    pub end_time: f32,
}

impl VehicleLight {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03133000)?;
        let start_time = r.f32()?;
        let end_time = r.f32()?;

        r.chunk_id(0x03133001)?;
        let _target = r.u32()?;

        Ok(Self {
            start_time,
            end_time,
        })
    }
}

/// Editing cut media block.
#[derive(Clone, Debug)]
pub struct EditingCut;

impl EditingCut {
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
pub struct DirtyLens {
    /// Keys of the media block.
    pub keys: Vec<key::DirtyLens>,
}

impl DirtyLens {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03165000)?;
        r.u32()?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let intensity = r.f32()?;

            Ok(key::DirtyLens { time, intensity })
        })?;

        Ok(Self { keys })
    }
}

/// Color grading media block.
#[derive(Clone)]
pub struct ColorGrading {
    /// Optional reference to the grade image file.
    pub grade: Option<InternalFileRef>,
    /// Keys of the media block.
    pub keys: Vec<key::ColorGrading>,
}

impl ColorGrading {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03186000)?;
        let grade = r.optional_internal_file_ref()?;

        r.chunk_id(0x03186001)?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let intensity = r.f32()?;

            Ok(key::ColorGrading { time, intensity })
        })?;

        Ok(Self { grade, keys })
    }
}

/// Manialink inferface media block.
#[derive(Clone)]
pub struct ManialinkInterface {
    /// Start time of the block in seconds. [0.0, ∞)
    pub start_time: f32,
    /// End time of the block in seconds. [0.0, ∞)
    pub end_time: f32,
    /// The manialink interface.
    pub manialink: String,
}

impl ManialinkInterface {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03195000)?;
        r.u32()?;
        let start_time = r.f32()?;
        let end_time = r.f32()?;
        r.u32()?;
        let manialink = r.string()?;

        Ok(Self {
            start_time,
            end_time,
            manialink,
        })
    }
}

/// Fog media block.
#[derive(Clone)]
pub struct Fog {
    /// Keys of the media block.
    pub keys: Vec<key::Fog>,
}

impl Fog {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x03199000)?;
        r.u32()?;
        let keys = r.list(|r| {
            let time = r.f32()?;
            let intensity = r.f32()?;
            let sky_intensity = r.f32()?;
            let distance = r.f32()?;
            r.f32()?;
            let red = r.f32()?;
            let green = r.f32()?;
            let blue = r.f32()?;
            let cloud_opacity = r.f32()?;
            let cloud_speed = r.f32()?;

            Ok(key::Fog {
                time,
                intensity,
                sky_intensity,
                distance,
                color: Rgb { red, green, blue },
                cloud_opacity,
                cloud_speed,
            })
        })?;

        Ok(Self { keys })
    }
}

/// Entity media block.
#[derive(Clone)]
pub struct Entity;

impl Entity {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read + Seek,
        I: BorrowMut<reader::IdState>,
        N: BorrowMut<reader::NodeState>,
    {
        r.chunk_id(0x0329F000)?;
        let version = r.u32()?;
        r.node(0x0911F000, EntityRecord::read)?;
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

/// Visibility of a opponent visibility media block.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug, TryFromInteger)]
#[non_exhaustive]
#[repr(u32)]
pub enum Visibility {
    #[default]
    Hidden,
    Ghost,
    Opaque,
}

/// Opponent visibility media block.
#[derive(Clone, Debug)]
pub struct OpponentVisibility {
    /// Start time of the block in seconds. [0.0, ∞)
    pub start_time: f32,
    /// End time of the block in seconds. [0.0, ∞)
    pub end_time: f32,
    /// Opponent visibility.
    pub visibility: Visibility,
}

impl OpponentVisibility {
    pub(crate) fn read<R, I, N>(r: &mut Reader<R, I, N>) -> ReadResult<Self>
    where
        R: Read,
    {
        r.chunk_id(0x0338B000)?;
        let start_time = r.f32()?;
        let end_time = r.f32()?;

        r.chunk_id(0x0338B001)?;
        let visibility = Visibility::try_from(r.u32()?).unwrap();

        Ok(Self {
            start_time,
            end_time,
            visibility,
        })
    }
}
