extern crate env_logger;
#[macro_use]
extern crate log;
extern crate siege_render;
extern crate winit;

use siege_render::{Config, Renderer, Tonemapper};
use std::sync::Arc;
use winit::EventsLoop;

fn main() {
    env_logger::init();

    let config = Config {
        major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch_version: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        vulkan_layers: vec![],
        asset_path: From::from("./examples"),
        tonemapper: Tonemapper::Clamp,
        ..Default::default()
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

    let _renderer = Renderer::new(config, arc_window.clone()).unwrap();

    info!("Got a renderer.");
}
