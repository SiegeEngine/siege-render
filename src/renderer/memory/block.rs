
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use dacite::core::{DeviceMemory, MappedMemory};
use super::_stride;
use errors::*;

#[derive(Debug, Clone)]
pub struct Block {
    pub memory: DeviceMemory,
    pub mapped_memory: Arc<MappedMemory>,
    pub offset: u64,
    pub ptr: *mut u8,
    pub size: u64,
    pub memory_type_index: u32, // for deallocation, to find the right chunk vec
    pub host_visible: bool, // to know whether we can write to it from the host
    pub is_coherent: bool, // to determine if we need to flush
    pub freelist: Arc<RwLock<Vec<u64>>>,
    pub element_alignment: u64,
    pub dirty: Arc<AtomicBool>,
}

impl Drop for Block {
    fn drop(&mut self) {
        // Mark our offset in the freelist before we drop
        let mut freelist = self.freelist.write().unwrap();
        freelist.push(self.offset);
    }
}

impl Block {
    pub fn as_ptr<T>(&self) -> &mut T {
        let p: *mut T = self.ptr as *mut T;
        unsafe { &mut *p }
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn as_ptr_at_offset<T>(&self, offset: usize) -> &mut T {
        let stride = _stride(::std::mem::size_of::<T>(),
                             self.element_alignment as usize);
        unsafe {
            let p = self.ptr.offset((offset * stride) as isize) as *mut T;
            &mut *p
        }
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn write<T: Copy>(&self, data: &T, offset: Option<usize>)
                          -> Result<()>
    {
        let stride = _stride(::std::mem::size_of::<T>(),
                             self.element_alignment as usize);
        let offset = offset.unwrap_or(0_usize);
        assert!(stride * (offset + 1) <= self.size as usize);

        unsafe {
            let p = self.ptr.offset((offset * stride) as isize) as *mut T;
            *p = *data;
        }

        // mark dirty
        self.dirty.store(true, Ordering::Relaxed);

        Ok(())
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn write_array<T: Copy>(&self, data: &[T], offset: Option<usize>)
                                -> Result<()>
    {
        let stride = _stride(::std::mem::size_of::<T>(),
                             self.element_alignment as usize);
        let offset = offset.unwrap_or(0_usize);

        assert!(stride * (offset + data.len()) <= self.size as usize);

        // If we don't have gaps, we can use rust slices
        if stride == ::std::mem::size_of::<T>() {
            unsafe {
                let dest: &mut [T] = ::std::slice::from_raw_parts_mut(
                    self.ptr.offset((offset * stride) as isize) as *mut T,
                    data.len()
                );
                dest.copy_from_slice(data);
            }
        } else {
            // Note: we cannot use copy_from_slice() because slices in rust don't
            // have alignment pdding between the elements.
            for i in offset..offset + data.len() {
                unsafe {
                    let p = self.ptr.offset((i * stride) as isize) as *mut T;
                    *p = data[i];
                }
            }
        }

        // mark dirty
        self.dirty.store(true, Ordering::Relaxed);

        Ok(())
    }
}
