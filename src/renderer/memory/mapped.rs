
use dacite::core::MappedMemory;
use super::{Block, _stride};
use errors::*;

#[derive(Debug)]
pub struct Mapped {
    pub block: Block,
    pub mapping: MappedMemory,
}

impl Mapped {
    pub fn as_ptr<T>(&self) -> &mut T {
        let p: *mut T = self.mapping.as_ptr() as *mut T;
        unsafe { &mut *p }
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn as_ptr_at_offset<T>(&self, offset: usize) -> &mut T {
        let stride = _stride(::std::mem::size_of::<T>(),
                             self.block.element_alignment as usize);
        unsafe {
            let u8p = self.mapping.as_ptr() as *mut u8;
            let p = u8p.offset((offset * stride) as isize) as *mut T;
            &mut *p
        }
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn write<T: Copy>(&self, data: &T, offset: Option<usize>, flush: bool)
                          -> Result<()>
    {
        let stride = _stride(::std::mem::size_of::<T>(),
                             self.block.element_alignment as usize);
        let offset = offset.unwrap_or(0_usize);
        assert!(stride * (offset + 1) <= self.block.size as usize);

        unsafe {
            let u8p = self.mapping.as_ptr() as *mut u8;
            let p = u8p.offset((offset * stride) as isize) as *mut T;
            *p = *data;
        }

        if flush { self.flush()?; }

        Ok(())
    }

    // offset is measured in "count of T's plus alignment padding", not in bytes.
    pub fn write_array<T: Copy>(&self, data: &[T], offset: Option<usize>, flush: bool)
                                -> Result<()>
    {
        let stride = _stride(::std::mem::size_of::<T>(),
                             self.block.element_alignment as usize);
        let offset = offset.unwrap_or(0_usize);

        assert!(stride * (offset + data.len()) <= self.block.size as usize);

        // If we don't have gaps, we can use rust slices
        if stride == ::std::mem::size_of::<T>() {
            unsafe {
                let u8p = self.mapping.as_ptr() as *mut u8;
                let dest: &mut [T] = ::std::slice::from_raw_parts_mut(
                    u8p.offset((offset * stride) as isize) as *mut T,
                    data.len()
                );
                dest.copy_from_slice(data);
            }
        } else {
            // Note: we cannot use copy_from_slice() because slices in rust don't
            // have alignment pdding between the elements.
            let u8p = self.mapping.as_ptr() as *mut u8;
            for i in offset..offset + data.len() {
                unsafe {
                    let p = u8p.offset((i * stride) as isize) as *mut T;
                    *p = data[i];
                }
            }
        }

        if flush { self.flush()?; }

        Ok(())
    }

    pub fn flush(&self) -> Result<()>
    {
        if !self.block.is_coherent {
            self.mapping.flush(&None)?;
        }
        Ok(())
    }
}
