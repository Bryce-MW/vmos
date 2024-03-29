#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_option)]
#![feature(stdsimd)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(iter_advance_by)]
#![feature(decl_macro)]
#![feature(ptr_metadata)]
#![feature(extern_types)]
#![feature(format_args_nl)]
#![no_std]
#![no_main]

mod acpi;
mod interrupts;
mod util;
mod vga;
mod memory;

use core::{
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::Ordering,
    arch::asm
};

use crate::{
    acpi::find_pcie,
    interrupts::{create_glob_idt, sti},
    util::println,
    vga::VGA
};
use crate::memory::create_high_mem;

static HELLO: &str = "Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> !
{
    if (_start as usize) < isize::MAX as usize {
        // NOTE(bryce): We are in a lower half kernel, i.e. the bootloader just called us
        println!("Got lower-half control from bootloader");

        // TODO(bryce): Remove this eventually
        unsafe {
            create_glob_idt();
            sti();
        }

        create_high_mem();
    } else {
        // NOTE(bryce): We are in a higher half kernel so we can now play with memory
        println!("Got higher-half control from kernel");

        unsafe {
            create_glob_idt();
            sti();

            find_pcie();
        }

        println!("{}\nSomething else", HELLO);
    }

    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> !
{
    VGA.color.store(0x0e, Ordering::Relaxed);
    let _ = println!("\n{}", _info);

    loop {
        hlt()
    }
}

fn hlt()
{
    // IMPORTANT(bryce): We are assuming we are on x64 so this is always safe.
    unsafe { asm!("hlt", options(nomem, preserves_flags, nostack)) }
}
