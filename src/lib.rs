extern crate ash;
extern crate ddsfile;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate separator;
#[macro_use]
extern crate serde_derive;
extern crate serde;

pub mod errors;
pub use self::errors::*;

pub mod config;
pub use self::config::Config;

pub mod renderer;

// These maximums are due to the size of memory chunks that we define in
// graphics/memory.rs.  4K resolution is the maximum that we support.
pub const MAX_HEIGHT: u32 = 2160;
pub const MAX_WIDTH: u32 = 3840;
