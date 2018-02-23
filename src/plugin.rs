
use dacite::core::CommandBuffer;
use errors::*;

pub trait Plugin {
    fn record_earlyz(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn record_shading(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn record_transparency(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn record_ui(&self, command_buffer: CommandBuffer) -> Result<()>;
}
