
mod requirements;
//use self::requirements::*;

mod setup;

use std::sync::Arc;

use dacite::core::Instance;
use dacite::ext_debug_report::DebugReportCallbackExt;
use dacite::khr_surface::SurfaceKhr;
use winit::Window;

use errors::*;
use config::Config;

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum VulkanLogLevel {
    Error,
    Warning,
    PerformanceWarning,
    Information,
    Debug
}

pub struct Renderer<S> {
    surface: SurfaceKhr,
    #[allow(dead_code)] // We don't use this directly, FFI uses it
    debug_callback: Option<DebugReportCallbackExt>,
    #[allow(dead_code)] // This must stay alive until we shut down
    instance: Instance,
    state: Arc<S>,
    window: Arc<Window>,
    config: Arc<Config>,
}

impl<S> Renderer<S> {
    pub fn new(config: Arc<Config>, window: Arc<Window>, state: Arc<S>)
               -> Result<Renderer<S>>
    {
        let instance = setup::setup_instance(&config, &window)?;

        let debug_callback = setup::setup_debug_callback(&config, &instance)?;

        let surface = setup::setup_surface(&window, &instance)?;

        Ok(Renderer {
            surface: surface,
            debug_callback: debug_callback,
            instance: instance,
            state: state,
            window: window,
            config: config
        })
    }
}
