use crate::Vec3;

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

#[derive(Clone, Debug)]
/// Bloom high dynamic range media block key.
pub struct BloomHdr {
    pub intensity: f32,
    pub streaks_intensity: f32,
    pub streaks_attenuation: f32,
}
