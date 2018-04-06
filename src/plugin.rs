
use dacite::core::{CommandBuffer, Extent2D};
use renderer::{Params, Stats};
use errors::*;

pub trait Plugin {
    fn record_geometry(&self, command_buffer: CommandBuffer);
    fn record_transparent(&self, command_buffer: CommandBuffer);
    fn record_ui(&self, command_buffer: CommandBuffer);
    fn update(&mut self, params: &mut Params, stats: &Stats) -> Result<()>;
    fn rebuild(&mut self, extent: Extent2D) -> Result<()>;
}
