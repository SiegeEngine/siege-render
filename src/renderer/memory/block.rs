
use std::sync::{Arc, RwLock};
use dacite::core::DeviceMemory;
use super::Mapped;
use errors::*;

#[derive(Debug, Clone)]
pub struct Block {
    pub memory: DeviceMemory,
    pub offset: u64,
    pub memory_type_index: u32, // for deallocation, to find the right chunk vec
    pub host_visible: bool, // to know whether we can write to it from the host
    pub is_coherent: bool, // to determine if we need to flush
    pub freelist: Arc<RwLock<Vec<u64>>>,
    pub size: u64,
    pub element_alignment: u64
}

impl Block {
    pub fn into_mapped(self) -> Result<Mapped>
    {
        use dacite::core::OptionalDeviceSize;

        // Must be host visible
        if !self.host_visible {
            return Err(ErrorKind::MemoryNotHostWritable.into());
        }

        let mapped_memory = self.memory.map(
            self.offset,
            OptionalDeviceSize::Size(self.size),
            Default::default())?;

        Ok(Mapped {
            block: self,
            mapping: mapped_memory,
        })
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        // Mark our offset in the freelist before we drop
        let mut freelist = self.freelist.write().unwrap();
        freelist.push(self.offset);
    }
}
