#![feature(asm)]
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
#![no_std]
#![no_main]

mod acpi;
mod interrupts;
mod util;
mod vga;

use core::{fmt::Write, panic::PanicInfo, sync::atomic::Ordering};

use crate::{
    acpi::find_pcie,
    interrupts::{create_glob_idt, sti},
    util::println,
    vga::VGA
};

static HELLO: &str = "Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> !
{
    unsafe {
        create_glob_idt();
        sti();

        find_pcie();
    }

    println!("{}\nSomething else", HELLO);

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
