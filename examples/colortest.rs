
#[macro_use] extern crate log;
#[macro_use] extern crate simple_logger;
extern crate siege_render;
extern crate winit;
extern crate dacite;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use winit::EventsLoop;

use dacite::core::{Pipeline, PipelineBindPoint,
                   CommandBuffer, PipelineLayout,
                   PrimitiveTopology, CullModeFlags, FrontFace,
                   Extent2D};
use siege_render::{Renderer, Pass, DepthHandling, BlendMode, Plugin,
                   Params, Stats, Config, Tonemapper, PipelineSetup};

pub fn main() {

    simple_logger::init().unwrap();

    let config = Config {
        major_version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
        minor_version: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
        patch_version: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        vulkan_layers: vec![],
        asset_path: From::from("./examples"),
        tonemapper: Tonemapper::Clamp,
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

    let mut renderer = Renderer::new(
        config, arc_window.clone(),
        resized.clone(), shutdown.clone()).unwrap();

    let colortest = Colortest::new(&mut renderer);

    renderer.plugin(Box::new(colortest));

    renderer.run();
}

pub struct Colortest {
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout,
}

impl Colortest {
    fn new(renderer: &mut Renderer) -> Colortest {
        let (pipeline_layout, pipeline) = renderer.create_pipeline(
            PipelineSetup {
                desc_set_layouts: vec![],
                vertex_shader: Some("colortest.vert"),
                vertex_shader_spec: None,
                fragment_shader: Some("colortest.frag"),
                fragment_shader_spec: None,
                vertex_type: None, // no vertex type
                topology: PrimitiveTopology::TriangleList,
                cull_mode: CullModeFlags::NONE,
                front_face: FrontFace::CounterClockwise,
                depth_handling: DepthHandling::None,
                blend: vec![BlendMode::Off],
                pass: Pass::Ui,
                push_constant_ranges: vec![]
            }).unwrap();

        Colortest {
            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
        }
    }
}

impl Plugin for Colortest {
    fn record_geometry(&self, _command_buffer: CommandBuffer) {
    }

    fn record_transparent(&self, command_buffer: CommandBuffer) {
        command_buffer.bind_pipeline(PipelineBindPoint::Graphics, &self.pipeline);
        command_buffer.draw(3, 1, 0, 0);
    }

    fn record_ui(&self, _command_buffer: CommandBuffer) {
    }

    fn update(&mut self, _params: &mut Params, _stats: &Stats)
              -> Result<bool, ::siege_render::Error>
    {
        Ok(false)
    }

    fn gpu_update(&mut self) -> Result<(), ::siege_render::Error>
    {
        Ok(())
    }

    fn rebuild(&mut self, _extent: Extent2D) -> Result<(), ::siege_render::Error> {
        Ok(())
    }
}
