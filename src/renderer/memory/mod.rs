
mod chunk;
pub use self::chunk::CHUNK_SIZE;

mod block;
pub use self::block::Block;

use std::collections::HashMap;
use std::fmt::{self, Display};
use separator::Separatable;
use dacite::core::{Device, PhysicalDeviceMemoryProperties,
                   PhysicalDeviceProperties,
                   MemoryRequirements, MemoryPropertyFlags,
                   BufferUsageFlags, MemoryType, DeviceMemory};

use errors::*;
use self::chunk::Chunk;

#[derive(Debug, Clone, Copy)]
pub enum Lifetime {
    Permanent,
    Temporary
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum Linearity {
    Linear = 0,
    Nonlinear = 1,
}
impl Display for Linearity {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        match *self {
            Linearity::Linear => write!(f, "Linear"),
            Linearity::Nonlinear => write!(f, "Nonlinear"),
        }
    }
}

// A Solo allocation stands alone. Some restrictions apply:
//  * It is never freed
//  * It is only for device memory; not mappable
//  * Intended for large render targets, but not limited as such
pub struct SoloInfo {
    pub memory_type_index: u32,
    pub memory_type: MemoryType, // for logging
    pub size: u64,
    pub reason: String,
}

pub struct Memory {
    // This maps from heap_index to the chunk set
    chunks: [HashMap<u32, Vec<Chunk>>; 2],
    memory_properties: PhysicalDeviceMemoryProperties,
    properties: PhysicalDeviceProperties,
    solos: Vec<SoloInfo>,
}

impl Memory {
    pub fn new(memory_properties: PhysicalDeviceMemoryProperties,
               properties: PhysicalDeviceProperties) -> Memory
    {
        info!("Max allocations: {}", properties.limits.max_memory_allocation_count);
        Memory {
            chunks: [HashMap::new(), HashMap::new()],
            memory_properties: memory_properties,
            properties: properties,
            solos: Vec::new(),
        }
    }

    pub fn allocate_solo_device_memory(
        &mut self,
        device: &Device,
        memory_requirements: &MemoryRequirements,
        memory_property_flags: MemoryPropertyFlags,
        reason: &str)
        -> Result<DeviceMemory>
    {
        use dacite::core::MemoryAllocateInfo;

        // Determine the memory_type we need, getting its index
        let memory_type_index = self.find_memory_type_index(
            memory_requirements.memory_type_bits, memory_property_flags);
        let memory_type_index = match memory_type_index {
            None => return Err(ErrorKind::OutOfGraphicsMemory.into()),
            Some(i) => i,
        };
        let memory_type = self.memory_properties.memory_types[memory_type_index as usize];

        let allocate_info = MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index: memory_type_index,
            chain: None,
        };
        let memory = device.allocate_memory(&allocate_info, None)?;

        let info = SoloInfo {
            memory_type_index: memory_type_index,
            memory_type: memory_type,
            size: memory_requirements.size,
            reason: reason.to_owned()
        };
        self.solos.push(info);

        Ok(memory)
    }

    pub fn allocate_device_memory(
        &mut self,
        device: &Device,
        memory_requirements: &MemoryRequirements,
        memory_property_flags: MemoryPropertyFlags,
        buffer_usage: Option<BufferUsageFlags>,
        linearity: Linearity,
        lifetime: Lifetime,
        reason: &str)
        -> Result<Block>
    {
        let l = linearity as usize;

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
        if ! self.chunks[l].contains_key(&memory_type_index) {
            self.chunks[l].insert(
                memory_type_index,
                vec![Chunk::new( &device, memory_type_index, memory_type )? ]
            );
        }

        let element_alignment = self.element_alignment(buffer_usage);

        // Get the chunk vector for the memory_type we care about
        let chunk_vec = self.chunks[l].get_mut(&memory_type_index).unwrap();

        // Try to allocate from each chunk in turn
        for chunk in chunk_vec.iter_mut() {
            if let Some(block) = chunk.allocate(
                memory_requirements.size,
                memory_requirements.alignment,
                element_alignment,
                lifetime,
                reason)
            {
                assert!(block.offset_in_chunk + block.size <= CHUNK_SIZE);
                return Ok(block);
            }
        }

        // Looks like we are going to need another chunk.
        let mut new_chunk = Chunk::new(
            &device, memory_type_index, memory_type)?;
        let block = new_chunk.allocate(
            memory_requirements.size,
            memory_requirements.alignment,
            element_alignment,
            lifetime,
            reason);
        chunk_vec.push(new_chunk);
        if let Some(block) = block {
            assert!(block.offset_in_chunk + block.size <= CHUNK_SIZE);
            Ok(block)
        } else {
            Err(ErrorKind::OutOfGraphicsMemory.into())
        }
    }

    pub fn log_usage(&self) {
        for solo in &self.solos {
            let mut propstring: String = String::new();
            if solo.memory_type.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL) {
                propstring.push_str("Device ");
            }
            if solo.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_VISIBLE) {
                propstring.push_str("Host ");
            }
            if solo.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_COHERENT) {
                propstring.push_str("HCoherent ");
            }
            if solo.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_CACHED) {
                propstring.push_str("HCached ");
            }
            if solo.memory_type.property_flags.contains(MemoryPropertyFlags::LAZILY_ALLOCATED) {
                propstring.push_str("Lazy ");
            }
            info!("type{} heap{}: {}",
                  solo.memory_type_index, solo.memory_type.heap_index, propstring);
            info!("  Solo  ({:>12}) Perm: {}", solo.size.separated_string(), solo.reason);
        }
        for (_, chunkvec) in &self.chunks[0] {
            for (i, chunk) in chunkvec.iter().enumerate() {
                chunk.log_usage(i, Linearity::Linear);
            }
        }
        for (_, chunkvec) in &self.chunks[1] {
            for (i, chunk) in chunkvec.iter().enumerate() {
                chunk.log_usage(i, Linearity::Nonlinear);
            }
        }
    }

    pub fn element_alignment(&self, buffer_usage: Option<BufferUsageFlags>)
                             -> u64
    {
        // Determine element_alignment
        let mut element_alignment = 1;
        if let Some(bu) = buffer_usage {
            if bu.contains(BufferUsageFlags::UNIFORM_BUFFER) {
                element_alignment = element_alignment.max(
                    self.properties.limits.min_uniform_buffer_offset_alignment);
            }
            if bu.contains(BufferUsageFlags::STORAGE_BUFFER) {
                element_alignment = element_alignment.max(
                    self.properties.limits.min_storage_buffer_offset_alignment);
            }
            if bu.contains(BufferUsageFlags::UNIFORM_TEXEL_BUFFER) {
                element_alignment = element_alignment.max(
                    self.properties.limits.min_uniform_buffer_offset_alignment);
                element_alignment = element_alignment.max(
                    self.properties.limits.min_texel_buffer_offset_alignment);
            }
            if bu.contains(BufferUsageFlags::STORAGE_TEXEL_BUFFER) {
                element_alignment = element_alignment.max(
                    self.properties.limits.min_storage_buffer_offset_alignment);
                element_alignment = element_alignment.max(
                    self.properties.limits.min_texel_buffer_offset_alignment);
            }
        }
        element_alignment
    }

    pub fn stride(&self, size_one: usize, buffer_usage: Option<BufferUsageFlags>)
                  -> usize
    {
        let element_alignment = self.element_alignment(buffer_usage);

        _stride(size_one, element_alignment as usize)
    }

    // This only flushes dirty chunks
    pub fn flush(&self) -> Result<()> {
        for (_, chunkvec) in &self.chunks[0] {
            for (_, chunk) in chunkvec.iter().enumerate() {
                chunk.flush()?;
            }
        }
        for (_, chunkvec) in &self.chunks[1] {
            for (_, chunk) in chunkvec.iter().enumerate() {
                chunk.flush()?;
            }
        }
        Ok(())
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

fn _stride(size_one: usize, element_alignment: usize)
           -> usize
{
    if element_alignment<=1 {
        size_one
    } else {
        element_alignment * (1 + ( (size_one-1)/element_alignment ) )
    }
}
