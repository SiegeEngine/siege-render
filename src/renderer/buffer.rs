
use errors::*;
use std::io::{Write, Read};
use dacite::core::{Buffer, Device, BufferUsageFlags, MemoryPropertyFlags,
                   BufferCopy, OptionalDeviceSize, Format, BufferView,
                   BufferViewCreateInfo};
use super::memory::{Memory, Block, Lifetime, Linearity};
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
            Linearity::Linear,
            lifetime,
            reason)?
    };

    buffer.bind_memory(block.memory.clone(), block.offset_in_chunk)?;

    Ok((buffer, block))
}

#[derive(Debug)]
pub struct HostVisibleBuffer {
    buffer: Buffer,
    block: Block,
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

        Ok(HostVisibleBuffer {
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

    pub fn as_ptr<T>(&self) -> Option<&mut T> {
        self.block.as_ptr()
    }

    pub fn as_ptr_at_offset<T>(&self, offset: usize) -> Option<&mut T> {
        self.block.as_ptr_at_offset(offset)
    }

    pub fn write_one<T: Copy>(&mut self, data: &T, offset: Option<usize>)
                          -> Result<()>
    {
        self.block.write_one(data, offset)
    }

    pub fn write_array<T: Copy>(&mut self, data: &[T], offset: Option<usize>)
                                -> Result<()>
    {
        self.block.write_array(data, offset)
    }
}

impl Write for HostVisibleBuffer {
    fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize>
    {
        self.block.write(buf)
    }

    fn flush(&mut self) -> ::std::io::Result<()>
    {
        self.block.flush()
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
        staging_buffer: &mut HostVisibleBuffer,
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
        staging_buffer.write_array::<T>(data, None)?;

        // Force a flush (FIXME if block held arc to mapped memory we would not have
        // to flush every chunk)
        memory.flush()?;

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

    // Warning! this does not align data on the device, it copies byte-for-byte.
    pub fn new_from_reader<R: Read>(
        device: &Device,
        memory: &mut Memory,
        commander: &Commander,
        src: &mut R,
        staging_buffer: &mut HostVisibleBuffer,
        usage: BufferUsageFlags,
        lifetime: Lifetime,
        reason: &str) -> Result<DeviceLocalBuffer>
    {
        // Copy data into staging buffer
        let size: u64 = ::std::io::copy(src, staging_buffer)?;

        // Create device buffer
        let device_buffer = {
            let (buffer, block) = _new(
                device, memory, size,
                usage | BufferUsageFlags::TRANSFER_DST,
                lifetime, reason, MemoryPropertyFlags::DEVICE_LOCAL)?;
            DeviceLocalBuffer {
                buffer: buffer,
                block: block
            }
        };

        // Force a flush (FIXME if block held arc to mapped memory we would not have
        // to flush every chunk)
        memory.flush()?;

        // Copy from staging buffer to device buffer
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
        staging_buffer.block.write_one(data, 0)?;

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

    pub fn get_buffer_view(&self, device: &Device, format: Format) -> Result<BufferView>
    {
        Ok(device.create_buffer_view(
            &BufferViewCreateInfo {
                flags: Default::default(),
                buffer: self.buffer.clone(),
                format: format,
                offset: 0,
                range: OptionalDeviceSize::WholeSize,
                chain: None,
            },
            None
        )?)
    }
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
