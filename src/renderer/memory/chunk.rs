
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use errors::*;
use dacite::core::{Device, DeviceMemory, MappedMemory, MemoryType, MemoryPropertyFlags,
                   OptionalDeviceSize};
use super::block::Block;
use super::Lifetime;

// We aim to target graphics cards with only 256 MB (which is only 244.14 MiB).
// So we can't go too crazy with the chunk size.
//
// Full 4K screen (3840x2160) at 64bpp is 66355200.0
// For 4K support, we need this for the shading image attachment. So memory chunk
// cannot be smaller than this.
//
// This is just slightly smaller than a true 64 MB, so we use 64 MB chunks.
//
pub const CHUNK_SIZE: u64 = 64 * 1048576; // 64 MB.

#[inline]
pub fn align_up(offset: u64, alignment: u64) -> u64 {
    ((offset + alignment).saturating_sub(1)) & !(alignment.wrapping_sub(1))
}

#[inline]
pub fn align_down(offset: u64, alignment: u64) -> u64 {
    offset & !(alignment.wrapping_sub(1))
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
    pub mapped_memory: Option<MappedMemory>,
    pub blocks: Vec<BlockInfo>, // keep these in order
    // List of block offsets which have dropped.
    pub freelist: Arc<RwLock<Vec<u64>>>,
    pub memory_type_index: u32,
    pub memory_type: MemoryType, // for logging
    pub start_of_perm: u64, // top of the free region, beyond which are PERM objects
    pub perm_blocks: Vec<BlockInfo>, // order is from top down, as they come
    pub dirty: Arc<AtomicBool>,
}

impl Chunk {
    /// Create a new chunk by asking Vulkan for more memory in the given
    /// memory_type index.
    pub fn new(device: &Device, memory_type_index: u32, memory_type: MemoryType)
               -> Result<Chunk>
    {
        use dacite::core::MemoryAllocateInfo;

        let allocate_info = MemoryAllocateInfo {
            allocation_size: CHUNK_SIZE,
            memory_type_index: memory_type_index,
            chain: None,
        };
        let memory = device.allocate_memory(&allocate_info, None)?;

        let mapped_memory = if memory_type.property_flags.contains(
            MemoryPropertyFlags::HOST_VISIBLE)
        {
            Some(memory.map(0, OptionalDeviceSize::WholeSize, Default::default())?)
        } else {
            None
        };

        Ok(Chunk {
            memory: memory,
            mapped_memory: mapped_memory,
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
    pub fn allocate(&mut self, size: u64, alignment: u64,
                    element_alignment: u64, lifetime: Lifetime, reason: &str)
                    -> Option<Block>
    {
        match lifetime {
            Lifetime::Permanent =>
                self.allocate_perm(size, alignment, element_alignment, reason),
            Lifetime::Temporary =>
                self.allocate_normal(size, alignment, element_alignment, reason)
        }
    }

    fn allocate_perm(&mut self, size: u64, alignment: u64,
                     element_alignment: u64,
                     reason: &str)
                     -> Option<Block>
    {
        let offset = if let Some(unaligned_offset) = self.start_of_perm.checked_sub(size) {
            align_down(unaligned_offset, alignment)
        } else {
            return None;
        };

        if let Some(lastblock) = self.blocks.last() {
            if lastblock.end() > offset {
                return None
            }
        }

        self.start_of_perm = offset;
        Some(self.make_block(offset, size, element_alignment, reason, None))
    }

    fn allocate_normal(&mut self, size: u64, alignment: u64,
                       element_alignment: u64,
                       reason: &str)
                    -> Option<Block>
    {
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

    fn make_block(&mut self, offset: u64, size: u64,
                  element_alignment: u64, reason: &str,
                  insert_block_at: Option<usize>) -> Block
    {
        use dacite::core::MemoryPropertyFlags;

        let ptr = match self.mapped_memory {
            None => None,
            Some(ref mm) => Some( unsafe {
                (mm.as_ptr() as *mut u8).offset(offset as isize)
            } )
        };

        let block = Block {
            memory: self.memory.clone(),
            offset_in_chunk: offset,
            ptr: ptr,
            size: size,
            stdio_write_offset: 0,
            memory_type_index: self.memory_type_index,
            is_coherent: self.memory_type.property_flags.contains(
                MemoryPropertyFlags::HOST_COHERENT),
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

    /// Log info messages about memory usage
    pub fn log_usage(&self, chunk_number: usize) {

        if chunk_number == 0 {
            // New type of chunk, log details about the memory type

            let mut propstring: String = String::new();
            if self.memory_type.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL) {
                propstring.push_str("Device ");
            }
            if self.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_VISIBLE) {
                propstring.push_str("Host ");
            }
            if self.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_COHERENT) {
                propstring.push_str("HCoherent ");
            }
            if self.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_CACHED) {
                propstring.push_str("HCached ");
            }
            if self.memory_type.property_flags.contains(MemoryPropertyFlags::LAZILY_ALLOCATED) {
                propstring.push_str("Lazy ");
            }

            info!("T{}, heap{}, {}: ",
                  self.memory_type_index,
                  self.memory_type.heap_index,
                  propstring);
        }

        for block in &self.perm_blocks {
            info!("  C{} PERM size={:9} ({:2.0}%): {}",
                  chunk_number,
                  block.size,
                  (block.size * 100) as f32 / CHUNK_SIZE as f32,
                  block.reason);
        }
        for block in &self.blocks {
            info!("  C{} TEMP size={:9} ({:2.0}%): {}",
                  chunk_number,
                  block.size,
                  (block.size * 100) as f32 / CHUNK_SIZE as f32,
                  block.reason);
        }
    }

    pub fn flush(&self) -> Result<()>
    {
        // only if something is mapped
        if let Some(ref mm) = self.mapped_memory {
            // Coherent memory does not need explicit flushes
            if ! self.memory_type.property_flags.contains(MemoryPropertyFlags::HOST_COHERENT) {
                // Only flush if dirty
                if self.dirty.load(Ordering::Relaxed) {
                    mm.flush(&None)?;

                    // and reset the dirty bit
                    self.dirty.store(false, Ordering::Relaxed);
                }
            }
        }
        Ok(())
    }
}
