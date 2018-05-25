use ash::version::V1_0;
use ash::vk::types::DebugReportCallbackEXT;
use ash::{Entry, Instance};
use config::Config;
use errors::*;
use math::{Mat4, Vec4};
use plugin::Plugin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use winit::Window;

mod setup;
use self::setup::*;

mod stats;
pub use self::stats::{Stats, Timings};

#[derive(Debug, Deserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum VulkanLogLevel {
    Error,
    Warning,
    PerformanceWarning,
    Information,
    Debug,
}

// Passes that consumers of the library can plug into
pub enum Pass {
    Geometry,
    Transparent,
    Ui,
}

pub enum DepthHandling {
    None,
    Some(bool, bool), // test, write
}

pub enum BlendMode {
    Off,
    Alpha,
    PreMultiplied,
    Add,
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

#[repr(u32)]
pub enum Timestamp {
    FullStart = 0,
    FullEnd = 1,
    GeometryStart = 2,
    GeometryEnd = 3,
    ShadingStart = 4,
    ShadingEnd = 5,
    TransparentStart = 6,
    TransparentEnd = 7,
    Blur1Start = 8,
    Blur1End = 9,
    Blur2Start = 10,
    Blur2End = 11,
    PostStart = 12,
    PostEnd = 13,
    UiStart = 14,
    UiEnd = 15,
}
const TS_QUERY_COUNT: u32 = 16;

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

pub struct Renderer {
    plugins: Vec<Box<Plugin>>,
    debug_report_callback: DebugReportCallbackEXT,
    instance: Instance<V1_0>,
    entry: Entry<V1_0>,
    shutdown: Arc<AtomicBool>,
    resized: Arc<AtomicBool>,
    stats: Stats,
    window: Arc<Window>,
    config: Config,
}

impl Renderer {
    pub fn new(
        config: Config,
        window: Arc<Window>,
        resized: Arc<AtomicBool>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Renderer> {
        let entry = Entry::new()?;

        let instance = setup_instance(&entry, &config, &window)?;

        let debug_report_callback = setup_debug_report(&entry, &config, &instance)?;

        Ok(Renderer {
            plugins: Vec::new(),
            debug_report_callback: debug_report_callback,
            instance: instance,
            entry: entry,
            shutdown: shutdown.clone(),
            resized: resized.clone(),
            stats: Default::default(),
            window: window.clone(),
            config: config,
        })
    }

    pub fn plugin(&mut self, plugin: Box<Plugin>) -> Result<()> {
        self.plugins.push(plugin);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        unimplemented!()
    }
}
