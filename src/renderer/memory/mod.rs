
mod chunk;
pub use self::chunk::CHUNK_SIZE;

mod block;
pub use self::block::Block;

use std::collections::HashMap;
use dacite::core::{Device, PhysicalDeviceMemoryProperties,
                   PhysicalDeviceProperties,
                   MemoryRequirements, MemoryPropertyFlags};

use errors::*;
use self::chunk::Chunk;

#[derive(Debug, Clone, Copy)]
pub enum Lifetime {
    Permanent,
    Temporary
}

pub struct Memory {
    // This maps from heap_index to the chunk set
    chunks: HashMap<u32, Vec<Chunk>>,
    memory_properties: PhysicalDeviceMemoryProperties,
//    max_memory_allocation_count: u32,
}

impl Memory {
    pub fn new(memory_properties: PhysicalDeviceMemoryProperties,
               properties: &PhysicalDeviceProperties) -> Memory
    {
        info!("Max allocations: {}", properties.limits.max_memory_allocation_count);
        Memory {
            chunks: HashMap::new(),
            memory_properties: memory_properties,
//            max_memory_allocation_count: properties.limits.max_memory_allocation_count,
        }
    }

    pub fn allocate_device_memory(
        &mut self,
        device: &Device,
        memory_requirements: &MemoryRequirements,
        memory_property_flags: MemoryPropertyFlags,
        lifetime: Lifetime,
        reason: &str)
        -> Result<Block>
    {
        // If the required memory is higher than the chunk size, abandon all hope
        if memory_requirements.size > CHUNK_SIZE {
            panic!("Memory requested is greater than chunk size: {} > {}",
                   memory_requirements.size, CHUNK_SIZE);
        }

        // Determine the memory_type we need, getting its index
        let memory_type_index = self.find_memory_type_index(
            memory_requirements.memory_type_bits, memory_property_flags);
        let memory_type_index = match memory_type_index {
            None => return Err(ErrorKind::OutOfGraphicsMemory.into()),
            Some(i) => i,
        };
        let memory_type = self.memory_properties.memory_types[memory_type_index as usize];

        // If we have not allocated this type of memory before, we have to setup
        // a new Chunk vector for it:
        if ! self.chunks.contains_key(&memory_type_index) {
            self.chunks.insert(
                memory_type_index,
                vec![Chunk::new( &device, memory_type_index, memory_type )? ]
            );
        }

        // Get the chunk vector for the memory_type we care about
        let chunk_vec = self.chunks.get_mut(&memory_type_index).unwrap();

        // Try to allocate from each chunk in turn
        for chunk in chunk_vec.iter_mut() {
            if let Some(block) = chunk.allocate(
                memory_requirements.size,
                memory_requirements.alignment,
                lifetime,
                reason)
            {
                assert!(block.offset + block.size <= CHUNK_SIZE);
                return Ok(block);
            }
        }

        // Looks like we are going to need another chunk.
        let mut new_chunk = Chunk::new(
            &device, memory_type_index, memory_type)?;
        let block = new_chunk.allocate(
            memory_requirements.size,
            memory_requirements.alignment,
            lifetime,
            reason);
        chunk_vec.push(new_chunk);
        if let Some(block) = block {
            assert!(block.offset + block.size <= CHUNK_SIZE);
            Ok(block)
        } else {
            Err(ErrorKind::OutOfGraphicsMemory.into())
        }
    }

    pub fn log_usage(&self) {
        for (_, chunkvec) in &self.chunks {
            for (i,chunk) in chunkvec.iter().enumerate() {
                chunk.log_usage(i);
            }
        }
    }

    fn find_memory_type_index(
        &self,
        memory_type_bits: u32,
        flags: MemoryPropertyFlags)
        -> Option<u32>
    {

        // Try to find an exactly matching memory flag
        let best_suitable_index =
            self._find_memory_type_index_f(
                memory_type_bits, flags,
                |property_flags, flags| property_flags == flags);

        if best_suitable_index.is_some() {
            return best_suitable_index;
        }

        // Otherwise find a memory flag that works
        self._find_memory_type_index_f(
            memory_type_bits, flags,
            |property_flags, flags| property_flags & flags == flags)
    }

    fn _find_memory_type_index_f<F>(
        &self,
        mut memory_type_bits: u32,
        flags: MemoryPropertyFlags,
        f: F)
        -> Option<u32>
        where F: Fn(MemoryPropertyFlags, MemoryPropertyFlags) -> bool
    {
        for (index, ref memory_type) in self.memory_properties.memory_types.iter().enumerate() {
            if index >= self.memory_properties.memory_types.len() as usize {
                return None;
            }
            if memory_type_bits & 1 == 1 {
                if f(memory_type.property_flags, flags) {
                    return Some(index as u32);
                }
            }
            memory_type_bits = memory_type_bits >> 1;
        }
        None
    }
}
