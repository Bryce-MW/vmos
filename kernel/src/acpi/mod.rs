use core::ops::Range;
use core::ptr::slice_from_raw_parts_mut;
use crate::util::{bytes_to_elements, k_bytes, OffsetBytes};

pub unsafe fn find_pcie() {
    // NOTE(bryce): The EBDA base address downshifted by 4 is usually found at 0x40E. It is 16 bits.
    let ebda = (*(0x40E as *mut u16) << 4) as *mut u64;
    let rsdp_range = slice_from_raw_parts_mut(ebda, bytes_to_elements::<u64>(k_bytes(1)));
}
