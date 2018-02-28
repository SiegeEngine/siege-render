
use std::borrow::Borrow;
use dacite::core::{CommandBuffer, Extent2D};
use errors::*;

pub trait Plugin {
    fn record_earlyz(&self, command_buffer: CommandBuffer);
    fn record_opaque(&self, command_buffer: CommandBuffer);
    fn record_transparent(&self, command_buffer: CommandBuffer);
    fn record_ui(&self, command_buffer: CommandBuffer);
    fn update(&mut self) -> Result<()>;
    fn rebuild(&mut self, extent: Extent2D) -> Result<()>;
}

pub trait DPlugin: Borrow<Plugin> { }
