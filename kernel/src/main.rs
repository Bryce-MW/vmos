#![feature(asm)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(core_intrinsics)]

#![no_std]
#![no_main]

mod vga;

use core::panic::PanicInfo;
use crate::vga::{test_panic, VGA};
use core::fmt::Write;
use core::sync::atomic::Ordering;

static HELLO: &str = "Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write!(VGA.writer(), "{}\nSomething else", HELLO).unwrap();

    test_panic();

    loop {pause()}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    VGA.color.store(0x0e, Ordering::Relaxed);
    let _ = write!(VGA.writer(), "\n{}\n", _info);

    loop {pause()}
}

fn pause() {
    // IMPORTANT(bryce): We are assuming we are on x64 so this is always safe.
    unsafe {
        asm!("pause")
    }
}
