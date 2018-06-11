use ash::version::V1_0;
use ash::vk::types::{DebugReportCallbackEXT, SurfaceKHR};
use ash::{Device, Entry, Instance};
use config::Config;
use errors::*;
use plugin::Plugin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use winit::Window;

mod queue_indices;

mod memory;
use self::memory::Memory;

mod requirements;
pub use self::requirements::DeviceRequirements;

mod setup;
use self::setup::physical::Physical;

mod stats;
pub use self::stats::{Stats, Timings};

mod types;
pub use self::types::*;

pub struct Renderer {
    plugins: Vec<Box<Plugin>>,

    #[allow(dead_code)] // FIXME, check again later
    memory: Memory,
    #[allow(dead_code)] // FIXME, check again later
    device: Device<V1_0>,
    #[allow(dead_code)] // FIXME, check again later
    physical: Physical,
    #[allow(dead_code)] // FIXME, check again later
    debug_report_callback: DebugReportCallbackEXT,
    #[allow(dead_code)] // FIXME, check again later
    surface_khr: SurfaceKHR,
    #[allow(dead_code)] // FIXME, check again later
    instance: Instance<V1_0>,
    #[allow(dead_code)] // FIXME, check again later
    entry: Entry<V1_0>,
    #[allow(dead_code)] // FIXME, check again later
    shutdown: Arc<AtomicBool>,
    #[allow(dead_code)] // FIXME, check again later
    resized: Arc<AtomicBool>,
    #[allow(dead_code)] // FIXME, chech again later
    stats: Stats,
    #[allow(dead_code)] // FIXME, check again later
    window: Arc<Window>,
    #[allow(dead_code)] // FIXME, check again later
    config: Config,
}

impl Renderer {
    pub fn new(
        config: Config,
        requirements: DeviceRequirements,
        window: Arc<Window>,
        resized: Arc<AtomicBool>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<Renderer> {
        let entry = Entry::new()?;

        let instance = self::setup::instance::setup_instance(&entry, &config, &window)?;

        let debug_report_callback =
            self::setup::debug_report::setup_debug_report(&entry, &config, &instance)?;

        let surface_khr = self::setup::surface::setup_surface(&entry, &instance, &window)?;

        let physical = self::setup::physical::find_suitable_device(
            &entry,
            &instance,
            surface_khr,
            &requirements,
        )?;

        let device = self::setup::device::create_device(&instance, &physical, &requirements)?;

        let memory = Memory::new(physical.memory_properties.clone(),
                                 physical.properties.clone());

        Ok(Renderer {
            plugins: Vec::new(),
            memory: memory,
            device: device,
            physical: physical,
            debug_report_callback: debug_report_callback,
            surface_khr: surface_khr,
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
