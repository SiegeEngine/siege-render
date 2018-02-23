
#![recursion_limit = "1024"]

// serialization
#[macro_use]
extern crate serde_derive;
extern crate serde;

// errors
#[macro_use]
extern crate error_chain;

// logging
#[macro_use]
extern crate log;

// graphics
extern crate vks;
extern crate dacite;
extern crate dacite_winit;
extern crate winit;
extern crate siege_mesh;

// files
extern crate ddsfile;
extern crate zstd;

// win32
#[cfg(windows)] extern crate user32;
#[cfg(windows)] extern crate winapi;

// math
extern crate siege_math;

// time
extern crate chrono;

// These maximums are due to the size of memory chunks that we define in
// graphics/memory.rs.  4K resolution is the maximum that we support.
pub const MAX_WIDTH: u32 = 3840;
pub const MAX_HEIGHT: u32 = 2160;

pub mod errors;
pub use errors::*;

pub mod config;
pub use config::Config;

pub mod renderer;
pub use renderer::{Renderer, ImageWrap, SiegeBuffer, HostVisibleBuffer,
                   DeviceLocalBuffer};

pub mod vertex;
pub use vertex::{VulkanVertex, ColoredVertex, StandardVertex, GuiRectangleVertex,
                 GrayboxVertex, CheapV1Vertex, CheapV2Vertex, StarVertex, CubemapVertex};

pub mod format;

pub mod plugin;
pub use plugin::Plugin;
