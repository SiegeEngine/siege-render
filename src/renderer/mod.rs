
use errors::*;
use std::sync::Arc;
use config::Config;
use winit::Window;
use dacite::core::Instance;

mod requirements;
use self::requirements::*;

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
    state: Arc<S>,
    window: Arc<Window>,
    config: Arc<Config>,
}

impl<S> Renderer<S> {
    pub fn new(config: Arc<Config>, window: Arc<Window>, state: Arc<S>)
               -> Result<Renderer<S>>
    {
        Ok(Renderer {
            state: state,
            window: window,
            config: config
        })
    }
}
