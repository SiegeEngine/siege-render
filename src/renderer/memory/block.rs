
use std::sync::{Arc, RwLock};
use errors::*;
use dacite::core::{DeviceMemory};

#[derive(Debug, Clone)]
pub struct Block {
    pub memory: DeviceMemory,
    pub offset: u64,
    pub size: u64,
    pub memory_type_index: u32, // for deallocation, to find the right chunk vec
    pub is_coherent: bool, // to determine if we need to flush
    pub freelist: Arc<RwLock<Vec<u64>>>,
}

impl Block {
    // Write any amount of data within the memory block, as long as it fits
    pub fn write<T: Copy>(&self, data: &[T], block_offset: u64) -> Result<()>
    {
        use dacite::core::OptionalDeviceSize;

        // Data must fit
        let thissize = ::std::mem::size_of_val(data) as u64;
        assert!(block_offset + thissize <= self.size);

        let mapped_memory = self.memory.map(
            self.offset + block_offset,
            OptionalDeviceSize::Size(thissize),
            Default::default())?;

        unsafe {
            let dest: &mut [T] = ::std::slice::from_raw_parts_mut(
                mapped_memory.as_ptr() as *mut T,
                data.len()
            );
            dest.copy_from_slice(data);
        }

        // If memory is not coherent, we need to tell vulkan to flush caches
        if !self.is_coherent {
            mapped_memory.flush(&None)?;
        }

        Ok(())
    }

    // Write any amount of data within the memory block, as long as it fits
    pub fn write_one<T: Copy>(&self, data: &T, block_offset: u64) -> Result<()>
    {
        use dacite::core::OptionalDeviceSize;

        // Data must fit
        let thissize = ::std::mem::size_of_val(data) as u64;
        assert!(block_offset + thissize <= self.size);

        let mapped_memory = self.memory.map(
            self.offset + block_offset,
            OptionalDeviceSize::Size(thissize),
            Default::default())?;

        unsafe {
            let dest: *mut T = mapped_memory.as_ptr() as *mut T;
            *dest = *data;
        }

        // If memory is not coherent, we need to tell vulkan to flush caches
        if !self.is_coherent {
            mapped_memory.flush(&None)?;
        }

        Ok(())
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        // Mark our offset in the freelist before we drop
        let mut freelist = self.freelist.write().unwrap();
        freelist.push(self.offset);
    }
}
