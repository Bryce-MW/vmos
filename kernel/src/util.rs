use core::{fmt::Write, mem::size_of};

use crate::vga::VGA;

pub trait OffsetBytes
{
    unsafe fn offset_bytes(self, count: isize) -> Self;
    fn wrapping_offset_bytes(self, count: isize) -> Self;
}

impl<T> OffsetBytes for *mut T
{
    unsafe fn offset_bytes(self, count: isize) -> Self { (self as *mut u8).offset(count) as Self }
    fn wrapping_offset_bytes(self, count: isize) -> Self
    {
        (self as *mut u8).wrapping_offset(count) as Self
    }
}
impl<T> OffsetBytes for *const T
{
    unsafe fn offset_bytes(self, count: isize) -> Self { (self as *const u8).offset(count) as Self }
    fn wrapping_offset_bytes(self, count: isize) -> Self
    {
        (self as *const u8).wrapping_offset(count) as Self
    }
}

pub struct PtrIter<T>(*const T, usize);
impl<T> Iterator for PtrIter<T>
{
    type Item = *const T;
    fn next(&mut self) -> Option<Self::Item>
    {
        if self.1 == 0 {
            None
        } else {
            self.1 -= 1;
            let res = self.0;
            self.0 = self.0.wrapping_add(1);
            Some(res)
        }
    }
}

pub trait IterPtr<T>
{
    fn iter_ptr(self) -> PtrIter<T>;
}
impl<T> IterPtr<T> for *const [T]
{
    fn iter_ptr(self) -> PtrIter<T> { PtrIter(self.as_ptr(), self.len()) }
}

pub const fn k_bytes(n: usize) -> usize { n * 1024 }

pub const fn bytes_to_elements<T>(bytes: usize) -> usize
{
    bytes.checked_div(size_of::<T>()).unwrap()
}

pub macro println($($arg:tt)*) {
    VGA.writer().write_fmt(format_args_nl!($($arg)*)).unwrap()
}

pub macro print($($arg:tt)*) {
    VGA.writer().write_fmt(format_args!($($arg)*)).unwrap()
}
