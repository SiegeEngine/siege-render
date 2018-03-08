
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use dacite::core::DeviceMemory;
use super::_stride;
use errors::*;

#[derive(Debug, Clone)]
pub struct Block {
    pub memory: DeviceMemory,
    pub ptr: Option<*mut u8>,
    pub offset: u64,
    pub size: u64,
    pub memory_type_index: u32, // for deallocation, to find the right chunk vec
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
    pub fn as_ptr<T>(&self) -> Option<&mut T> {
        // mark dirty, under the presumption that the caller will write
        self.dirty.store(true, Ordering::Relaxed);

        self.ptr.map(|rpu| {
            let rpt: *mut T = rpu as *mut T;
            unsafe { &mut *rpt }
        })
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn as_ptr_at_offset<T>(&self, offset: usize) -> Option<&mut T> {
        // mark dirty, under the presumption that the caller will write
        self.dirty.store(true, Ordering::Relaxed);

        self.ptr.map(|rpu| {
            let stride = _stride(::std::mem::size_of::<T>(),
                                 self.element_alignment as usize);
            unsafe {
                let rpt = rpu.offset((offset * stride) as isize) as *mut T;
                &mut *rpt
            }
        })
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn write<T: Copy>(&mut self, data: &T, offset: Option<usize>)
                          -> Result<()>
    {
        let ptr = match self.ptr {
            None => return Err(ErrorKind::MemoryNotHostWritable.into()),
            Some(rpu) => rpu
        };

        let stride = _stride(::std::mem::size_of::<T>(),
                             self.element_alignment as usize);
        let offset = offset.unwrap_or(0_usize);
        assert!(stride * (offset + 1) <= self.size as usize);

        unsafe {
            let p = ptr.offset((offset * stride) as isize) as *mut T;
            *p = *data;
        }

        // mark dirty
        self.dirty.store(true, Ordering::Relaxed);

        Ok(())
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn write_array<T: Copy>(&mut self, data: &[T], offset: Option<usize>)
                                -> Result<()>
    {
        let ptr = match self.ptr {
            None => return Err(ErrorKind::MemoryNotHostWritable.into()),
            Some(rpu) => rpu
        };

        let stride = _stride(::std::mem::size_of::<T>(),
                             self.element_alignment as usize);
        let offset = offset.unwrap_or(0_usize);

        assert!(stride * (offset + data.len()) <= self.size as usize);

        // If we don't have gaps, we can use rust slices
        if stride == ::std::mem::size_of::<T>() {
            unsafe {
                let dest: &mut [T] = ::std::slice::from_raw_parts_mut(
                    ptr.offset((offset * stride) as isize) as *mut T,
                    data.len()
                );
                dest.copy_from_slice(data);
            }
        } else {
            // Note: we cannot use copy_from_slice() because slices in rust don't
            // have alignment pdding between the elements.
            for i in offset..offset + data.len() {
                unsafe {
                    let p = ptr.offset((i * stride) as isize) as *mut T;
                    *p = data[i];
                }
            }
        }

        // mark dirty
        self.dirty.store(true, Ordering::Relaxed);

        Ok(())
    }
}
