use super::Rgb;
use crate::Vec3;

/// Media block effect key.
#[derive(Clone, Debug)]
pub struct Effect;

/// Color media block key.
#[derive(Clone, Debug)]
pub struct Color;

/// Time media block key.
#[derive(Clone, Debug)]
pub struct Time {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub time_value: f32,
    pub tangent: f32,
}

/// Custom camera media block key.
#[derive(Clone, Debug)]
pub struct CustomCamera;

/// Camera shake effect media block key.
#[derive(Clone, Debug)]
pub struct CameraShakeEffect {
    pub intensity: f32,
    pub speed: f32,
}

/// Music volume media block key.
#[derive(Clone, Debug)]
pub struct MusicVolume {
    pub music_volume: f32,
    pub sound_volume: f32,
}

/// Sound media block key.
#[derive(Clone, Debug)]
pub struct Sound {
    pub volume: f32,
    pub position: Vec3<f32>,
}

/// Transition fade media block key.
#[derive(Clone, Debug)]
pub struct TransitionFade {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub opacity: f32,
}

/// Depth of field fade media block key.
#[derive(Clone, Debug)]
pub struct DepthOfField {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub focus_distance: f32,
    pub lens_size: f32,
    pub target_position: Vec3<f32>,
}

/// Tone mapping media block key.
#[derive(Clone, Debug)]
pub struct ToneMapping {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub exposure: f32,
    pub max_hdr: f32,
    pub light_trail_scale: f32,
}

/// Bloom media block key.
#[derive(Clone, Debug)]
pub struct Bloom {
    pub intensity: f32,
    pub streaks_intensity: f32,
    pub streaks_attenuation: f32,
}

/// Time speed media block key.
#[derive(Clone, Debug)]
pub struct TimeSpeed {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub speed: f32,
}

/// Dirty lens media block key.
#[derive(Clone, Debug)]
pub struct DirtyLens {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub intensity: f32,
}

/// Color grading media block key.
#[derive(Clone, Debug)]
pub struct ColorGrading {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub intensity: f32,
}

/// Fog media block key.
#[derive(Clone, Debug)]
pub struct Fog {
    /// Time of the key in seconds. [0.0, ∞)
    pub time: f32,
    pub intensity: f32,
    pub sky_intensity: f32,
    pub distance: f32,
    /// Color of the fog.
    pub color: Rgb,
    pub cloud_opacity: f32,
    pub cloud_speed: f32,
}
