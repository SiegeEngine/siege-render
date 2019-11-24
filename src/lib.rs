
#![recursion_limit = "1024"]

#[macro_use] extern crate log;
#[macro_use] extern crate glsl_to_spirv_macros;
#[macro_use] extern crate glsl_to_spirv_macros_impl;

// These maximums are due to the size of memory chunks that we define in
// graphics/memory.rs.  4K resolution is the maximum that we support.
pub const MAX_WIDTH: u32 = 3840;
pub const MAX_HEIGHT: u32 = 2160;

pub mod error;
pub use crate::error::Error;

pub mod config;
pub use crate::config::Config;

pub mod renderer;
pub use crate::renderer::{Renderer, Pass, ImageWrap,
                   HostVisibleBuffer, DeviceLocalBuffer, VulkanMesh, Lifetime,
                   BlendMode, Params, Stats, Timings, Tonemapper, PipelineSetup};

pub mod vertex;
pub use crate::vertex::{VulkanVertex, ColoredVertex, StandardVertex, GuiRectangleVertex,
                 GrayboxVertex, CheapV1Vertex, CheapV2Vertex, StarVertex, CubemapVertex};

pub mod format;

pub mod plugin;
pub use crate::plugin::Plugin;
