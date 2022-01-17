use core::{
    fmt::Write,
    mem::size_of,
    ptr,
    ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
    str
};

use crate::{
    util::{bytes_to_elements, k_bytes, print, println, IterPtr},
    vga::VGA
};

pub unsafe fn find_pcie()
{
    let rsdp = find_rsdp();
    println!("RDSP: {:?}", *rsdp);
    assert_eq!((*rsdp).revision, 0); // Debug?
    // TODO(bryce): Check the checksums
    println!("RSDP Address: {:p}", rsdp);
    let rsdt = UnknownTable::rsdt((*rsdp).rsdt as *const _);
    println!("RSDT Address: {:p}", rsdt);
    println!("RSDT Head: {:?}", (*rsdt).header);
    println!("RSDT Length: {}", (*rsdt).pointer_to_other_sdt.len());
    for addr in &(*rsdt).pointer_to_other_sdt {
        println!("{:x}", *addr);
    }
}

const EBDA_PTR_PTR: *mut u16 = 0x40e as *mut u16;
const EBDA_SEARCH_LEN: usize = bytes_to_elements::<u64>(k_bytes(1));
const BIOS_RO_PTR: *mut u64 = 0xe0000 as *mut u64;
const BIOS_RO_SEARCH_LEN: usize = bytes_to_elements::<u64>(0x100000 - 0xe0000);
const RSDP_HEADER: u64 = u64::from_ne_bytes(*b"RSD PTR ");

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct RSDP
{
    signature: u64,
    checksum:  u8,
    oem_id:    [u8; 6],
    revision:  u8,
    // NOTE(bryce): 32 bit address
    rsdt:      u32
}

unsafe fn find_rsdp() -> *const RSDP
{
    // NOTE(bryce): The EBDA base address downshifted by 4 is usually found at
    //  0x40E. It is 16 bits.
    let ebda = ((*EBDA_PTR_PTR as usize) << 4) as *mut u64;
    let rsdp_range = slice_from_raw_parts_mut(ebda, EBDA_SEARCH_LEN);
    println!("elms: {}", EBDA_SEARCH_LEN);
    println!("ptr: {:p}", ebda);
    for ptr in rsdp_range.iter_ptr().step_by(2) {
        if *ptr == RSDP_HEADER {
            return ptr as *const RSDP;
        }
    }

    let bios_ro = slice_from_raw_parts_mut(BIOS_RO_PTR, BIOS_RO_SEARCH_LEN);
    for ptr in bios_ro.iter_ptr().step_by(2) {
        if *ptr == RSDP_HEADER {
            return ptr as *const RSDP;
        }
    }

    panic!("Could not find rsdp")
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct TableHeader
{
    signature:        u32,
    length:           u32,
    revision:         u8,
    checksum:         u8,
    oem_id:           [u8; 6],
    oem_table_id:     [u8; 8],
    oem_revision:     u32,
    creator_id:       u32,
    creator_revision: u32
}

extern "C" {
    type UnknownSlice;
}
#[repr(C, packed)]
struct UnknownTable
{
    header:               TableHeader,
    pointer_to_other_sdt: UnknownSlice
}
#[repr(C, packed)]
struct RSDT
{
    header:               TableHeader,
    pointer_to_other_sdt: [u32]
}

impl UnknownTable
{
    unsafe fn rsdt(s: *const UnknownTable) -> *const RSDT
    {
        let size = ((*s).header.length as usize - size_of::<TableHeader>()) / size_of::<u32>();
        ptr::from_raw_parts(s as *const (), size)
    }
}

const MCFG_SIG: u32 = u32::from_ne_bytes(*b"MCFG");
#[repr(C, packed)]
struct MCFG
{
    header: TableHeader
}
