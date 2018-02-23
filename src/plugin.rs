
use dacite::core::CommandBuffer;
use errors::*;

pub trait Plugin {
    fn record_earlyz(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn record_opaque(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn record_transparent(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn record_ui(&self, command_buffer: CommandBuffer) -> Result<()>;
    fn update(&mut self) -> Result<()>;
    fn upload(&mut self) -> Result<()>;
    fn rebuild(&mut self, width: u32, height: u32) -> Result<()>;
}
