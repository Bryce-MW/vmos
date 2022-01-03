use core::mem::size_of;

pub trait OffsetBytes {
    unsafe fn offset_bytes(self, count: isize) -> Self;
    fn wrapping_offset_bytes(self, count: isize) -> Self;
}

impl<T> OffsetBytes for *mut T {
    unsafe fn offset_bytes(self, count: isize) -> Self {
        (self as *mut u8).offset(count) as Self
    }
    fn wrapping_offset_bytes(self, count: isize) -> Self {
        (self as *mut u8).wrapping_offset(count) as Self
    }
}
impl<T> OffsetBytes for *const T {
    unsafe fn offset_bytes(self, count: isize) -> Self {
        (self as *const u8).offset(count) as Self
    }
    fn wrapping_offset_bytes(self, count: isize) -> Self {
        (self as *const u8).wrapping_offset(count) as Self
    }
}

pub const fn k_bytes(n: usize) -> usize {
    n * 1024
}

pub const fn bytes_to_elements<T>(bytes: usize) -> usize {
    bytes.checked_div(size_of::<T>()).unwrap()
}
