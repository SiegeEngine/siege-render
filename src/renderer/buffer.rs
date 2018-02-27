
use errors::*;
use dacite::core::{Buffer, Device, BufferUsageFlags, MemoryPropertyFlags,
                   BufferCopy};
use super::memory::{Memory, Block, Mapped, Lifetime};
use super::commander::Commander;

fn _new(
    device: &Device,
    memory: &mut Memory,
    size: u64,
    usage: BufferUsageFlags,
    lifetime: Lifetime,
    reason: &str,
    flags: MemoryPropertyFlags)
    -> Result<(Buffer, Block)>
{
    let buffer = {
        use dacite::core::{BufferCreateInfo, SharingMode};
        let create_info = BufferCreateInfo {
            flags: Default::default(),
            size: size,
            usage: usage,
            sharing_mode: SharingMode::Exclusive,
            queue_family_indices: vec![ ], // FIXME when sharing_mode is concurrent
            chain: None,
        };
        device.create_buffer(&create_info, None)?
    };

    let block = {
        let memory_requirements = buffer.get_memory_requirements();
        memory.allocate_device_memory(
            device,
            &memory_requirements,
            flags,
            Some(usage),
            lifetime,
            reason)?
    };

    buffer.bind_memory(block.memory.clone(), block.offset)?;

    Ok((buffer, block))
}

#[derive(Debug)]
pub struct HostVisibleBuffer {
    buffer: Buffer,
    mapped: Mapped,
}

impl HostVisibleBuffer {
    // Creates a new HostVisibleBuffer. The type T is used for sizing (including alignment
    // padding) but is not part of the created type, so you can use it with
    // other types later.
    pub fn new<T>(
        device: &Device,
        memory: &mut Memory,
        count: usize,
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<HostVisibleBuffer>
    {
        let stride = memory.stride(::std::mem::size_of::<T>(), Some(usage));
        let size = (count * stride) as u64;

        let (buffer, block) = _new(device, memory, size, usage, lifetime, reason,
                                   MemoryPropertyFlags::HOST_VISIBLE)?;
        let mapped = block.into_mapped()?;

        Ok(HostVisibleBuffer {
            buffer: buffer,
            mapped: mapped,
        })
    }

    pub fn inner(&self) -> Buffer {
        self.buffer.clone()
    }

    pub fn size(&self) -> u64 {
        self.mapped.block.size
    }

    pub fn as_ptr<T>(&self) -> &mut T {
        self.mapped.as_ptr()
    }

    pub fn as_ptr_at_offset<T>(&self, offset: usize) -> &mut T {
        self.mapped.as_ptr_at_offset(offset)
    }

    pub fn write<T: Copy>(&self, data: &T, offset: Option<usize>, flush: bool)
                          -> Result<()>
    {
        self.mapped.write(data, offset, flush)
    }

    pub fn write_array<T: Copy>(&self, data: &[T], offset: Option<usize>, flush: bool)
                                -> Result<()>
    {
        self.mapped.write_array(data, offset, flush)
    }

    pub fn flush(&self) -> Result<()> {
        self.mapped.flush()
    }
}

#[derive(Debug, Clone)]
pub struct DeviceLocalBuffer {
    buffer: Buffer,
    block: Block,
}

impl DeviceLocalBuffer {
    // Creates a new DeviceLocalBuffer. The type T is used for sizing (including alignment
    // padding) but is not part of the created type, so you can use it with
    // other types later.
    pub fn new<T>(
        device: &Device,
        memory: &mut Memory,
        count: usize,
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<DeviceLocalBuffer>
    {
        let stride = memory.stride(::std::mem::size_of::<T>(), Some(usage));
        let size = (count * stride) as u64;

        let (buffer, block) = _new(device, memory, size, usage, lifetime, reason,
                                   MemoryPropertyFlags::DEVICE_LOCAL)?;

        Ok(DeviceLocalBuffer {
            buffer: buffer,
            block: block,
        })
    }

    pub fn inner(&self) -> Buffer {
        self.buffer.clone()
    }

    pub fn size(&self) -> u64 {
        self.block.size
    }

    pub fn new_uploaded<T: Copy>(
        device: &Device,
        memory: &mut Memory,
        commander: &Commander,
        staging_buffer: &HostVisibleBuffer,
        data: &[T],
        mut usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<DeviceLocalBuffer>
    {
        let stride = memory.stride(::std::mem::size_of::<T>(), Some(usage));
        let size = (data.len() * stride) as u64;
        assert!(size <= staging_buffer.size());

        // Create the device buffer
        usage |= BufferUsageFlags::TRANSFER_DST;
        let device_buffer = Self::new::<T>(device, memory, data.len(), usage, lifetime, reason)?;

        // Write the data to the staging buffer
        staging_buffer.write_array::<T>(data, None, true)?;

        // Copy the data through
        copy(device, commander,
             &staging_buffer.buffer,
             &device_buffer.buffer,
             &[BufferCopy {
                 src_offset: 0,
                 dst_offset: 0,
                 size: size,
             }])?;

        Ok(device_buffer)
    }

    /*
    pub fn upload(
        &self,
        device: &Device,
        commander: &Commander
        staging_buffer: &HostVisibleBuffer<u8>,
        data: &[T],
        offset: u64)
        -> Result<()>
    {
        let size = ::std::mem::size_of_val(data) as u64;
        assert!(size <= staging_buffer.size);
        assert!(offset + size <= self.size);

        // Write the data to the staging buffer
        staging_buffer.block.write(data, 0)?;

        // Copy the data through
        staging_buffer.copy_to_buffer(
            device,
            commander,
            &self.buffer,
            &[BufferCopy {
                src_offset: 0,
                dst_offset: offset,
                size: size,
            }])?;

        Ok(())
    }
     */
}

fn copy(
    device: &Device,
    commander: &Commander,
    src: &Buffer,
    dest: &Buffer,
    regions: &[BufferCopy])
    -> Result<()>
{
    use dacite::core::{CommandBufferResetFlags, CommandBufferBeginInfo,
                       CommandBufferUsageFlags,
                       FenceCreateInfo, FenceCreateFlags, Fence, Timeout,
                       SubmitInfo, PipelineStageFlags};

    commander.xfr_command_buffer.reset(CommandBufferResetFlags::RELEASE_RESOURCES)?;

    let command_buffer_begin_info = CommandBufferBeginInfo {
        flags: CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        inheritance_info: None,
        chain: None
    };
    commander.xfr_command_buffer.begin(&command_buffer_begin_info)?;

    commander.xfr_command_buffer.copy_buffer(
        src, dest, regions);

    commander.xfr_command_buffer.end()?;

    let fence = {
        let create_info = FenceCreateInfo {
            flags: FenceCreateFlags::empty(),
            chain: None
        };
        device.create_fence(&create_info, None)?
    };

    let submit_info = SubmitInfo {
        wait_semaphores: vec![],
        wait_dst_stage_mask: vec![PipelineStageFlags::BOTTOM_OF_PIPE], // comes after TRANSFER
        command_buffers: vec![commander.xfr_command_buffer.clone()],
        signal_semaphores: vec![],
        chain: None
    };
    Fence::reset_fences(&[fence.clone()])?;
    commander.xfr_queue.submit( Some(&[submit_info]), Some(&fence) )?;
    Fence::wait_for_fences(&[fence], true, Timeout::Infinite)?;
    Ok(())
}
