#![feature(asm)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(stdsimd)]
#![no_std]
#![no_main]

mod interrupts;
mod vga;

use core::{arch::x86_64::ud2, fmt::Write, panic::PanicInfo, sync::atomic::Ordering};

use crate::{
    interrupts::{create_glob_idt, sti},
    vga::VGA
};

static HELLO: &str = "Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> !
{
    unsafe {
        create_glob_idt();
        sti();
    }

    write!(VGA.writer(), "{}\nSomething else", HELLO).unwrap();

    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> !
{
    VGA.color.store(0x0e, Ordering::Relaxed);
    let _ = writeln!(VGA.writer(), "\n{}", _info);

    loop {
        hlt()
    }
}

fn hlt()
{
    // IMPORTANT(bryce): We are assuming we are on x64 so this is always safe.
    unsafe { asm!("hlt") }
}
