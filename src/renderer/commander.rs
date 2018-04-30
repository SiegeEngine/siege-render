
use dacite::core::{Device, Queue, CommandPool, CommandBuffer, CommandBufferLevel,
                   CommandBufferAllocateInfo};

use errors::*;
use super::setup::QueueIndices;

pub struct GfxCommandBuffers {
    pub pre: CommandBuffer,
    pub geom: Vec<CommandBuffer>,
    pub after_geom: CommandBuffer,
    pub transparent: Vec<CommandBuffer>,
    pub after_transparent: CommandBuffer,
    pub ui: Vec<CommandBuffer>,
    pub after_ui: CommandBuffer,
}

pub struct Commander {
    pub plugin_cmdbuffer_staleness: Vec<bool>,
    pub num_framebuffers: u32,
    pub gfx_queue: Queue,
    pub gfx_command_buffers: Vec<GfxCommandBuffers>, // one per swapchain target
    pub gfxutil_command_buffer: CommandBuffer,
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

        let gfxutil_command_buffer = {
            let allocate_info = CommandBufferAllocateInfo {
                command_pool: gfx_command_pool.clone(),
                level: CommandBufferLevel::Primary,
                command_buffer_count: 1,
                chain: None,
            };
            let mut cbs = CommandPool::allocate_command_buffers(&allocate_info)?;
            cbs.pop().unwrap()
        };

        let gfx_command_buffers = {
            let mut cbs = {
                let allocate_info = CommandBufferAllocateInfo {
                    command_pool: gfx_command_pool.clone(),
                    level: CommandBufferLevel::Primary,
                    command_buffer_count: 4 * num_framebuffers,
                    chain: None,
                };
                CommandPool::allocate_command_buffers(&allocate_info)?
            };

            let mut gfxcb: Vec<GfxCommandBuffers> = vec![];
            for _ in 0..num_framebuffers {
                gfxcb.push(GfxCommandBuffers {
                    pre: cbs.pop().unwrap(),
                    geom: vec![],
                    after_geom: cbs.pop().unwrap(),
                    transparent: vec![],
                    after_transparent: cbs.pop().unwrap(),
                    ui: vec![],
                    after_ui: cbs.pop().unwrap(),
                });
            }
            gfxcb
        };

        let gfx_queue = device.get_queue(queue_indices.graphics_family,
                                         queue_indices.graphics_index);

        Ok(Commander {
            plugin_cmdbuffer_staleness: vec![],
            num_framebuffers: num_framebuffers,
            gfx_queue: gfx_queue,
            gfx_command_buffers: gfx_command_buffers,
            gfxutil_command_buffer: gfxutil_command_buffer,
            gfx_command_pool: gfx_command_pool,
            xfr_queue: xfr_queue,
            xfr_command_buffer: xfr_command_buffer,
            xfr_command_pool: xfr_command_pool,
        })
    }

    pub fn one_more_plugin(&mut self) -> Result<()>
    {
        let allocate_info = CommandBufferAllocateInfo {
            command_pool: self.gfx_command_pool.clone(),
            level: CommandBufferLevel::Primary,
            command_buffer_count: 3 * self.num_framebuffers,
            chain: None,
        };
        let mut cbs = CommandPool::allocate_command_buffers(&allocate_info)?;
        for si in 0..self.num_framebuffers as usize {
            self.gfx_command_buffers[si].geom.push(cbs.pop().unwrap());
            self.gfx_command_buffers[si].transparent.push(cbs.pop().unwrap());
            self.gfx_command_buffers[si].ui.push(cbs.pop().unwrap());
        }

        self.plugin_cmdbuffer_staleness.push(true);

        Ok(())
    }
}
