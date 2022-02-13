use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use crate::memory::MemMapFlag::Usable;
use crate::println;
use crate::util::{gb, kb, mb};

const MEM_MAP_LOC: *mut MemMapEntry<u32> = 0x3000 as _;
const PML4_LOC: *mut PageTable = 0x1000 as _;
const FREE_TABLES_LOC: *mut [PageTable; 4] = 0x4000 as _;

#[repr(C, packed)]
struct PageTable
{

}

#[derive(Copy, Clone)]
#[repr(C)]
struct MemMapEntry<T>
{
    base: u64,
    len: u64,
    flags: T,
    _reserved: u32
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
        let mut len = 0usize;
        while (*MEM_MAP_LOC.add(len)).base < u64::MAX {
            (*MEM_MAP_LOC.add(len)).flags = MemMapFlag::from((*MEM_MAP_LOC.add(len)).flags) as u32;
            len += 1;
        }
        let mem_map = slice_from_raw_parts_mut(MEM_MAP_LOC as *mut MemMapEntry<MemMapFlag>, len);

        let mut s4k = 0u64;
        let mut s2m = 0u64;
        let mut s1g = 0u64;

        for entry in (*mem_map).iter() {
            println!("{:016x} {:019x} {:?}", entry.base, entry.len, entry.flags);

            if entry.flags == Usable {
                let mut p0 = *entry;
                let mut p1 = MemMapEntry {
                    base: entry.base + entry.len,
                    len: 0,
                    flags: MemMapFlag::Bad,
                    _reserved: 0,
                };

                if entry.len >= gb!(1) {
                    let start = (entry.base & !(gb!(1) - 1)) + if entry.base & (gb!(1) - 1) != 0 { gb!(1) } else { 0 };
                    let size = entry.len - (start - entry.base) & !(gb!(1) - 1);
                    s1g += size / gb!(1);

                    p0.len = start - entry.base;
                    p1.base = start + size;
                    p1.len = (entry.base + entry.len) - start + size;

                    println!("({:x} {:x}), {}, ({:x}, {:x})", p0.base, p0.len, size / gb!(1), p1.base, p1.len);
                }

                if p1.len >= mb!(1) {
                    let base = p1.base;
                    let len = p1.len;

                    let size = len & !(mb!(1) - 1);
                    s2m += size / mb!(1);

                    p1.base = base + size;
                    p1.len = len - size;

                    println!("{}, ({:x}, {:x})", size / mb!(1), p1.base, p1.len);
                }

                if p0.len >= mb!(1) {
                    let base = p0.base;
                    let len = p0.len;

                    let start = (base & !(mb!(1) - 1)) + if base & (mb!(1) - 1) != 0 { mb!(1) } else { 0 };
                    let size = len - (start - base) & !(mb!(1) - 1);
                    s2m += size / mb!(1);

                    p0.len = start - base;
                    let right = start + size - (base + len);

                    if right != 0 {
                        p1.base = start + size;
                        p1.len = right;
                        println!("({:x} {:x}), {}, ({:x}, {:x})", p0.base, p0.len, size / mb!(1), p1.base, p1.len);
                    } else {
                        println!("({:x} {:x}), {}", p0.base, p0.len, size / mb!(1));
                    }
                }

                s4k += p0.len / kb!(4) + p1.len / kb!(4);
            }
        }

        println!("4k: {}, 2M: {}, 1G: {}", s4k, s2m, s1g);
    }
}
