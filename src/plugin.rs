
use dacite::core::{CommandBuffer, Extent2D};
use renderer::Params;
use errors::*;

pub trait Plugin {
    fn record_earlyz(&self, command_buffer: CommandBuffer);
    fn record_opaque(&self, command_buffer: CommandBuffer);
    fn record_transparent(&self, command_buffer: CommandBuffer);
    fn record_ui(&self, command_buffer: CommandBuffer);
    fn update(&mut self, &mut Params) -> Result<()>;
    fn rebuild(&mut self, extent: Extent2D) -> Result<()>;
}
