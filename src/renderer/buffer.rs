
use errors::*;
use std::marker::PhantomData;
use dacite::core::{Buffer, Device, BufferUsageFlags, MemoryPropertyFlags,
                   BufferCopy};
use super::memory::{Memory, Block, Lifetime};
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
            lifetime,
            reason)?
    };

    buffer.bind_memory(block.memory.clone(), block.offset)?;

    Ok((buffer, block))
}

#[derive(Debug, Clone)]
pub struct HostVisibleBuffer<T: Copy> {
    pub buffer: Buffer,
    pub block: Block,
    pub size: u64, // this is the size of the data.  block.size might be padded out.
    _phantom: PhantomData<T>, // we don't actually keep the data in here.
}

impl<T: Copy> HostVisibleBuffer<T> {
    pub fn new(
        device: &Device,
        memory: &mut Memory,
        size: u64,
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<HostVisibleBuffer<T>>
    {
        let (buffer, block) = _new(device, memory, size, usage, lifetime, reason,
                                   MemoryPropertyFlags::HOST_VISIBLE)?;

        Ok(HostVisibleBuffer {
            buffer: buffer,
            block: block,
            size: size,
            _phantom: PhantomData,
        })
    }

    pub fn inner<'a>(&'a self) -> &'a Buffer {
        &self.buffer
    }

    /*
    pub fn new_with_data(
        device: &Device,
        memory: &mut Memory,
        data: &[T],
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<HostVisibleBuffer<T>>
    {
        let size = ::std::mem::size_of_val(data) as u64;

        let output = Self::new(device, memory, size, usage, lifetime, reason)?;

        output.block.write(data, 0)?;

        Ok(output)
    }
     */

    pub fn new_with_single_data(
        device: &Device,
        memory: &mut Memory,
        data: &T,
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<HostVisibleBuffer<T>>
    {
        let size = ::std::mem::size_of_val(data) as u64;

        let output = Self::new(device, memory, size, usage, lifetime, reason)?;

        output.block.write_one(data, 0)?;

        Ok(output)
    }

    fn copy_to_buffer(
        &self,
        device: &Device,
        commander: &Commander,
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
            self.inner(),
            dest,
            regions);

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
}

#[derive(Debug, Clone)]
pub struct DeviceLocalBuffer<T: Copy> {
    pub buffer: Buffer,
    pub block: Block,
    pub size: u64, // this is the size of the data.  block.size might be padded out.
    _phantom: PhantomData<T>, // we don't actually keep the data in here.
}

impl<T: Copy> DeviceLocalBuffer<T> {
    pub fn new(
        device: &Device,
        memory: &mut Memory,
        size: u64,
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<DeviceLocalBuffer<T>>
    {
        let (buffer, block) = _new(device, memory, size, usage, lifetime, reason,
                                   MemoryPropertyFlags::DEVICE_LOCAL)?;

        Ok(DeviceLocalBuffer {
            buffer: buffer,
            block: block,
            size: size,
            _phantom: PhantomData,
        })
    }

    pub fn inner<'a>(&'a self) -> &'a Buffer {
        &self.buffer
    }

    pub fn new_uploaded(
        device: &Device,
        memory: &mut Memory,
        commander: &Commander,
        staging_buffer: &HostVisibleBuffer<u8>,
        data: &[T],
        mut usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<DeviceLocalBuffer<T>>
    {
        let size = ::std::mem::size_of_val(data) as u64;
        assert!(size <= staging_buffer.size);

        // Create the device buffer
        usage |= BufferUsageFlags::TRANSFER_DST;
        let device_buffer = Self::new(device, memory, size, usage, lifetime, reason)?;

        // Write the data to the staging buffer
        staging_buffer.block.write(data, 0)?;

        // Copy the data through
        staging_buffer.copy_to_buffer(
            device,
            commander,
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
