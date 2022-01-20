use crate::println;

const MEM_MAP_LOC: *const MemMapEntry = 0x3000 as _;

#[repr(C)]
struct MemMapEntry
{
    base: u64,
    len: u64,
    flags: u32,
    _reserved: u32
}

#[derive(Debug)]
#[repr(u32)]
enum MemMapFlag {
    Unknown = 0,
    Usable = 1,
    Reserved = 2,
    ACPI = 3,
    NVS = 4,
    Bad = 5
}

impl From<u32> for MemMapFlag {
    fn from(flag: u32) -> Self {
        use MemMapFlag::*;
        match flag {
            1 => Usable,
            2 => Reserved,
            3 => ACPI,
            4 => NVS,
            5 => Bad,
            _ => Unknown
        }
    }
}

pub fn create_high_mem() {
    unsafe {
        for i in 0..20 {
            let mem = MEM_MAP_LOC.offset(i as isize);
            println!("{:016x} {:016x} {:?}", (*mem).base, (*mem).len, MemMapFlag::from((*mem).flags));
        }
    }
}
