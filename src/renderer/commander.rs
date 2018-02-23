
use dacite::core::{Device, Queue, CommandPool, CommandBuffer};

use errors::*;
use super::setup::QueueIndices;

pub struct Commander {
    pub gfx_queue: Queue,
    pub gfx_command_buffers: Vec<CommandBuffer>,
    pub gfx_command_pool: CommandPool,
    pub xfr_queue: Queue,
    pub xfr_command_buffer: CommandBuffer,
    pub xfr_command_pool: CommandPool,
}

impl Commander {
    pub fn new(
        device: &Device,
        queue_indices: &QueueIndices,
        num_framebuffers: u32)
        -> Result<Commander>
    {
        let xfr_command_pool = {
            use dacite::core::{CommandPoolCreateInfo, CommandPoolCreateFlags};

            let create_info = CommandPoolCreateInfo {
                flags: CommandPoolCreateFlags::TRANSIENT |
                CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                queue_family_index: queue_indices.transfer_family,
                chain: None,
            };
            device.create_command_pool(&create_info, None)?
        };

        let xfr_command_buffer = {
            use dacite::core::{CommandBufferAllocateInfo, CommandBufferLevel};

            let allocate_info = CommandBufferAllocateInfo {
                command_pool: xfr_command_pool.clone(),
                level: CommandBufferLevel::Primary,
                command_buffer_count: 1,
                chain: None,
            };

            let mut cbs = CommandPool::allocate_command_buffers(&allocate_info)?;
            cbs.pop().unwrap()
        };

        let xfr_queue = device.get_queue(queue_indices.transfer_family,
                                         queue_indices.transfer_index);

        let gfx_command_pool = {
            use dacite::core::{CommandPoolCreateInfo, CommandPoolCreateFlags};

            let create_info = CommandPoolCreateInfo {
                flags: CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                queue_family_index: queue_indices.graphics_family,
                chain: None,
            };
            device.create_command_pool(&create_info, None)?
        };

        let gfx_command_buffers = {
            use dacite::core::{CommandBufferAllocateInfo, CommandBufferLevel};

            let allocate_info = CommandBufferAllocateInfo {
                command_pool: gfx_command_pool.clone(),
                level: CommandBufferLevel::Primary,
                // we allocate 1 extra for gfx_command_buffer, and pop it off
                command_buffer_count: num_framebuffers,
                chain: None,
            };
            CommandPool::allocate_command_buffers(&allocate_info)?
        };

        let gfx_queue = device.get_queue(queue_indices.graphics_family,
                                         queue_indices.graphics_index);

        Ok(Commander {
            gfx_queue: gfx_queue,
            gfx_command_buffers: gfx_command_buffers,
            gfx_command_pool: gfx_command_pool,
            xfr_queue: xfr_queue,
            xfr_command_buffer: xfr_command_buffer,
            xfr_command_pool: xfr_command_pool,
        })
    }
}
