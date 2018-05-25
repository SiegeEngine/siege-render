#![recursion_limit = "1024"]

#[macro_use]
extern crate ash;
#[cfg(feature = "cgmath")]
extern crate cgmath;
extern crate ddsfile;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[cfg(feature = "nalgebra")]
extern crate nalgebra;
extern crate separator;
#[cfg(feature = "siege-math")]
extern crate siege_math;
extern crate siege_mesh;
extern crate siege_vulkan;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[cfg(windows)]
extern crate winapi;
extern crate winit;

pub mod errors;
pub use self::errors::*;

pub mod config;
pub use self::config::Config;

pub mod format;

pub mod vertex;
pub use vertex::{CheapV1Vertex, CheapV2Vertex, ColoredVertex, CubemapVertex, GrayboxVertex,
                 GuiRectangleVertex, StandardVertex, StarVertex};

pub mod math;
pub use self::math::*;

pub mod plugin;
pub use plugin::Plugin;

pub mod renderer;
pub use renderer::{BlendMode, Params, Pass, Renderer, Stats, Timings, Tonemapper, VulkanLogLevel};

// These maximums are due to the size of memory chunks that we define in
// graphics/memory.rs.  4K resolution is the maximum that we support.
pub const MAX_HEIGHT: u32 = 2160;
pub const MAX_WIDTH: u32 = 3840;
