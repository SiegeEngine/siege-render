use math::{Mat4, Vec4};

#[derive(Debug, Deserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum VulkanLogLevel {
    Error,
    Warning,
    PerformanceWarning,
    Information,
    Debug,
}

#[repr(u32)]
#[derive(Debug, Deserialize, Clone, Copy)]
pub enum Tonemapper {
    Clamp = 0,
    Reinhard = 1,
    Exposure = 2,
    HybridLogGamma = 3,
    Falsecolor = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Params {
    /// The inverse of the projection matrix you are using. Generally the camera projection
    /// is not our business, and you should handle it via your plugins. However, our shading pass
    /// needs to reconstruct the view space position of fragments, and we require this matrix to
    /// do that.
    pub inv_projection: Mat4<f32>,

    /// The directions of the directional lights.
    /// FIXME: the limitation of "exactly 2 directional lights"
    pub dlight_directions: [Vec4<f32>; 2],

    /// The irradiances of the directional lights, in watter per square meter.
    /// FIXME: the limitation of "exactly 2 directional lights"
    pub dlight_irradiances: [Vec4<f32>; 2],

    /// The strength of the bloom effect. It should be a number in the range [0.0,1.0].
    /// 0.65 is the default.
    pub bloom_strength: f32,

    /// The cliff parameter for the bloom effect. It should be a number in the range
    /// [0.0,1.0].  0.7 is the default.
    pub bloom_cliff: f32,

    /// The blur level. This affects the entire screen. It should be a number in the
    /// range [0.0,1.0].  0.0 is the default.
    pub blur_level: f32,

    /// The level of ambient light, illumanance, measured in lux (lumens per square meter)
    pub ambient: f32,

    /// The luminance level, measured in nits (candela per square meter) which maps to a
    /// fully white pixel (prior to tone mapping)
    pub white_level: f32,

    /// The tonemapping algorithm to use
    pub tonemapper: Tonemapper,
}

pub struct Stats;
