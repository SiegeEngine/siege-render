use ash::version::V1_0;
use ash::vk::types::{DebugReportCallbackEXT, SurfaceKHR};
use ash::{Entry, Instance};
use config::Config;
use errors::*;
use plugin::Plugin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use winit::Window;

mod setup;
use self::setup::*;

mod stats;
pub use self::stats::{Stats, Timings};

mod types;
pub use self::types::*;

pub struct Renderer {
    plugins: Vec<Box<Plugin>>,

    debug_report_callback: DebugReportCallbackEXT,
    surface: SurfaceKHR,
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

        let surface = setup_surface(&entry, &instance, &window)?;

        Ok(Renderer {
            plugins: Vec::new(),
            debug_report_callback: debug_report_callback,
            surface: surface,
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
