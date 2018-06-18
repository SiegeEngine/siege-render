use ash::version::{DeviceV1_0, V1_0};
use ash::vk::types::{c_void, DeviceMemory, MappedMemoryRange, MemoryAllocateInfo, MemoryType,
                     StructureType, MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
                     MEMORY_PROPERTY_HOST_CACHED_BIT, MEMORY_PROPERTY_HOST_COHERENT_BIT,
                     MEMORY_PROPERTY_HOST_VISIBLE_BIT, MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT};
use ash::Device;
use errors::*;
use renderer::memory::block::Block;
use renderer::memory::{Lifetime, Linearity};
use separator::Separatable;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

pub const CHUNK_SIZE: u64 = 32 * 1048576; // 32 MB.

#[inline]
pub fn align_up(offset: u64, alignment: u64) -> u64 {
    ((offset + alignment).saturating_sub(1)) & !(alignment.wrapping_sub(1))
}

#[inline]
pub fn align_down(offset: u64, alignment: u64) -> u64 {
    offset & !(alignment.wrapping_sub(1))
}

pub struct Mapped {
    ptr: *mut c_void,
    offset: u64,
    size: u64,
}

pub struct BlockInfo {
    pub offset: u64,
    pub size: u64,
    pub reason: String,
}
impl BlockInfo {
    #[inline]
    pub fn end(&self) -> u64 {
        self.offset + self.size
    }
}

pub struct Chunk {
    pub memory: DeviceMemory,
    pub mapped_memory: Option<Mapped>,
    pub blocks: Vec<BlockInfo>, // keep these in order
    // List of block offsets which have dropped.
    pub freelist: Arc<RwLock<Vec<u64>>>,
    pub memory_type_index: u32,
    pub memory_type: MemoryType,     // for logging
    pub start_of_perm: u64,          // top of the free region, beyond which are PERM objects
    pub perm_blocks: Vec<BlockInfo>, // order is from top down, as they come
    pub dirty: Arc<AtomicBool>,
}

impl Chunk {
    /// Create a new chunk by asking Vulkan for more memory in the given
    /// memory_type index.
    pub fn new(
        device: &Device<V1_0>,
        memory_type_index: u32,
        memory_type: MemoryType,
    ) -> Result<Chunk> {
        const OFFSET: u64 = 0_u64;

        let mainfo = MemoryAllocateInfo {
            s_type: StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            allocation_size: CHUNK_SIZE,
            memory_type_index: memory_type_index,
        };

        let memory = unsafe { device.allocate_memory(&mainfo, None) }?;

        let mapped_memory_ptr = if memory_type
            .property_flags
            .intersects(MEMORY_PROPERTY_HOST_VISIBLE_BIT)
        {
            Some(unsafe { device.map_memory(memory, OFFSET, CHUNK_SIZE, Default::default()) }?)
        } else {
            None
        };

        let mapped = mapped_memory_ptr.map(|ptr| Mapped {
            ptr: ptr,
            offset: OFFSET,
            size: CHUNK_SIZE,
        });

        Ok(Chunk {
            memory: memory,
            mapped_memory: mapped,
            blocks: Vec::new(),
            freelist: Arc::new(RwLock::new(Vec::new())),
            memory_type_index: memory_type_index,
            memory_type: memory_type,
            start_of_perm: CHUNK_SIZE,
            perm_blocks: Vec::new(),
            dirty: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Allocate a block on the chunk, with the given size, alignment, and reason.
    /// element_alignment specifies alignment of each array element.
    /// If permanent, block is efficiently allocated and cannot be deallocated.
    pub fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
        element_alignment: u64,
        lifetime: Lifetime,
        reason: &str,
    ) -> Option<Block> {
        match lifetime {
            Lifetime::Permanent => self.allocate_perm(size, alignment, element_alignment, reason),
            Lifetime::Temporary => self.allocate_normal(size, alignment, element_alignment, reason),
        }
    }

    fn allocate_perm(
        &mut self,
        size: u64,
        alignment: u64,
        element_alignment: u64,
        reason: &str,
    ) -> Option<Block> {
        let offset = if let Some(unaligned_offset) = self.start_of_perm.checked_sub(size) {
            align_down(unaligned_offset, alignment)
        } else {
            return None;
        };

        if let Some(lastblock) = self.blocks.last() {
            if lastblock.end() > offset {
                return None;
            }
        }

        self.start_of_perm = offset;
        Some(self.make_block(offset, size, element_alignment, reason, None))
    }

    fn allocate_normal(
        &mut self,
        size: u64,
        alignment: u64,
        element_alignment: u64,
        reason: &str,
    ) -> Option<Block> {
        // Clear the freelist
        {
            let mut freelist = self.freelist.write().unwrap();

            // FIXME OPTIMAL: this is a slow operation, shifting parts
            // of the vector back.
            self.blocks.retain(|b| !freelist.contains(&b.offset));

            freelist.clear();
        }

        let mut p = 0;

        let num_blocks = self.blocks.len();

        let mut i = 0;
        while i < num_blocks {
            // If we have a big enough hole
            if self.blocks[i].offset > p + size {
                return Some(self.make_block(p, size, element_alignment, reason, Some(i)));
            } else {
                // move past the block
                p = align_up(self.blocks[i].offset + self.blocks[i].size, alignment);
            }
            i += 1;
        }
        if self.start_of_perm > p + size {
            Some(self.make_block(p, size, element_alignment, reason, Some(i)))
        } else {
            None
        }
    }

    fn make_block(
        &mut self,
        offset: u64,
        size: u64,
        element_alignment: u64,
        reason: &str,
        insert_block_at: Option<usize>,
    ) -> Block {
        let ptr = match self.mapped_memory {
            None => None,
            Some(ref mm) => Some(unsafe { (mm.ptr as *mut u8).offset(offset as isize) }),
        };

        let block = Block {
            memory: self.memory.clone(),
            offset_in_chunk: offset,
            ptr: ptr,
            size: size,
            stdio_write_offset: 0,
            memory_type_index: self.memory_type_index,
            is_coherent: self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_HOST_COHERENT_BIT),
            freelist: self.freelist.clone(),
            element_alignment: element_alignment,
            dirty: self.dirty.clone(),
        };

        let blockinfo = BlockInfo {
            offset: offset,
            size: size,
            reason: reason.to_owned(),
        };

        if let Some(pos) = insert_block_at {
            // Push into our blocks array
            // FIXME OPTIMAL: this is a slow operation for long vectors. Consider a
            //   different data structure
            self.blocks.insert(pos, blockinfo);
        } else {
            // was a permanent block
            self.perm_blocks.push(blockinfo);
        }

        block
    }

    pub fn flush(&self, device: &Device<V1_0>) -> Result<()> {
        // only if something is mapped
        if let Some(ref mm) = self.mapped_memory {
            // Coherent memory does not need explicit flushes
            if !self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_HOST_COHERENT_BIT)
            {
                // Only flush if dirty
                if self.dirty.load(Ordering::Relaxed) {
                    let range = MappedMemoryRange {
                        s_type: StructureType::MappedMemoryRange,
                        p_next: ptr::null(),
                        memory: self.memory,
                        offset: mm.offset,
                        size: mm.size,
                    };
                    unsafe { device.flush_mapped_memory_ranges(&[range]) }?;

                    // and reset the dirty bit
                    self.dirty.store(false, Ordering::Relaxed);
                }
            }
        }
        Ok(())
    }

    /// Log info messages about memory usage
    pub fn log_usage(&self, chunk_number: usize, linearity: Linearity) {
        if chunk_number == 0 {
            // New type of chunk, log details about the memory type

            let mut propstring: String = String::new();
            if self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_DEVICE_LOCAL_BIT)
            {
                propstring.push_str("Device ");
            }
            if self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_HOST_VISIBLE_BIT)
            {
                propstring.push_str("Host ");
            }
            if self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_HOST_COHERENT_BIT)
            {
                propstring.push_str("HCoherent ");
            }
            if self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_HOST_CACHED_BIT)
            {
                propstring.push_str("HCached ");
            }
            if self.memory_type
                .property_flags
                .intersects(MEMORY_PROPERTY_LAZILY_ALLOCATED_BIT)
            {
                propstring.push_str("Lazy ");
            }

            info!(
                "type{} heap{}: {} ({})",
                self.memory_type_index, self.memory_type.heap_index, propstring, linearity
            );
        }

        info!(
            "  Chunk {} ({})",
            chunk_number,
            CHUNK_SIZE.separated_string()
        );
        for block in &self.blocks {
            info!(
                "     size={:>12}      ({:2.0}%): {}",
                block.size.separated_string(),
                (block.size * 100) as f32 / CHUNK_SIZE as f32,
                block.reason
            );
        }
        for block in &self.perm_blocks {
            info!(
                "     size={:>12} Perm ({:2.0}%): {}",
                block.size.separated_string(),
                (block.size * 100) as f32 / CHUNK_SIZE as f32,
                block.reason
            );
        }
    }
}
