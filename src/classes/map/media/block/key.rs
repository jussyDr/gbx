use crate::Vec3;

/// Effect simi key.
#[derive(Clone, Debug)]
pub struct EffectSimi;

/// Fx colors media block key.
#[derive(Clone, Debug)]
pub struct FxColors;

/// Time media block key.
#[derive(Clone, Debug)]
pub struct Time {
    pub time: f32,
    pub time_value: f32,
    pub tangent: f32,
}

/// Camera custom media block key.
#[derive(Clone, Debug)]
pub struct CameraCustom;

/// Camera shake effect media block key.
#[derive(Clone, Debug)]
pub struct CameraShakeEffect {
    pub intensity: f32,
    pub speed: f32,
}

/// Music effect media block key.
#[derive(Clone, Debug)]
pub struct MusicEffect {
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
    pub time: f32,
    pub opacity: f32,
}

/// Depth of field fade media block key.
#[derive(Clone, Debug)]
pub struct DepthOfField {
    pub time: f32,
    pub focus_distance: f32,
    pub lens_size: f32,
    pub target_position: Vec3<f32>,
}

/// Tone mapping media block key.
#[derive(Clone, Debug)]
pub struct ToneMapping {
    pub time: f32,
    pub exposure: f32,
    pub max_hdr: f32,
    pub light_trail_scale: f32,
}

/// Bloom high dynamic range media block key.
#[derive(Clone, Debug)]
pub struct BloomHdr {
    pub intensity: f32,
    pub streaks_intensity: f32,
    pub streaks_attenuation: f32,
}

/// Time speed media block key.
#[derive(Clone, Debug)]
pub struct TimeSpeed {
    pub time: f32,
    pub speed: f32,
}

/// Dirty lens media block key.
#[derive(Clone, Debug)]
pub struct DirtyLens {
    pub time: f32,
    pub intensity: f32,
}

/// Color grading media block key.
#[derive(Clone, Debug)]
pub struct ColorGrading {
    pub time: f32,
    pub intensity: f32,
}

/// Fog media block key.
#[derive(Clone, Debug)]
pub struct Fog {
    pub time: f32,
    pub intensity: f32,
    pub sky_intensity: f32,
    pub distance: f32,
    /// RGB color of the fog.
    pub color: Vec3<f32>,
    pub cloud_opacity: f32,
    pub cloud_speed: f32,
}
