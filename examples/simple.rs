
#[macro_use] extern crate log;
#[macro_use] extern crate simple_logger;
extern crate siege_render;
extern crate winit;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use winit::EventsLoop;

use siege_render::{Config, Renderer};

pub fn main() {

    simple_logger::init().unwrap();

    let config = Config {
        major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch_version: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        vulkan_layers: vec![
            "VK_LAYER_LUNARG_standard_validation".to_owned()
        ],
        .. Default::default()
    };

    let events_loop = EventsLoop::new();

    let arc_window = {
        use winit::WindowBuilder;

        let mut builder = WindowBuilder::new()
            .with_title("Siege Render Example")
            .with_visibility(false) // will be turned on when graphics are ready
            .with_transparency(false)
            .with_dimensions(800, 600)
            .with_decorations(true);

        let window = builder.build(&events_loop).unwrap();

        Arc::new(window)
    };

    let resized = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));

    let renderer = Renderer::new(config, arc_window.clone(),
                                 resized.clone(), shutdown.clone())
        .unwrap();
}
