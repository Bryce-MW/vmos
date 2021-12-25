use core::cell::{Cell, UnsafeCell};
use core::fmt::Write;
use core::fmt;
use core::intrinsics::{offset, volatile_set_memory};
use core::ops::RangeBounds;
use core::ptr::{self, addr_of_mut, Unique};
use core::slice::SliceIndex;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::panic;

const VGA_COLS: usize = 80;
const VGA_ROWS: usize = 25;
const VGA_BUF_SIZE: usize = VGA_COLS * VGA_ROWS;
type VGABuf = [[u16; VGA_COLS]; VGA_ROWS];
pub static VGA: VGA = VGA {
    ptr: AtomicPtr::new(0xb8000 as *mut VGABuf),
    curr_row: AtomicUsize::new(0),
    curr_column: AtomicUsize::new(0)
};
pub struct VGA {
    ptr: AtomicPtr<VGABuf>,
    curr_row: AtomicUsize,
    curr_column: AtomicUsize
}

pub struct VGAWriter<'a>(&'a VGA);

impl VGA {
    pub fn writer(&self) -> VGAWriter {
        VGAWriter(&self)
    }
    unsafe fn clear(&self, buf: *mut VGABuf) {
        unsafe {
            // Clear the screen
            volatile_set_memory(buf, 0, 1);
            self.curr_row.store(0, Ordering::Release);
            self.curr_column.store(0, Ordering::Release);
        }
    }
    fn obtain(&self) -> *mut VGABuf {
        let mut res = self.ptr.swap(ptr::null_mut(), Ordering::Acquire);
        if res.is_null() {
            res = 0xb8000 as *mut VGABuf;
            // IMPORTANT(bryce): This can only happen on panic which must abort
            unsafe {
                self.clear(res);
            }
        }
        res
    }
    fn release(&self, buf: *mut VGABuf) {
        self.ptr.store(buf, Ordering::Release);
    }
    unsafe fn index(buf: *mut VGABuf, x: usize, y: usize) -> *mut u16 {
        unsafe {
            (*buf)[y].as_mut_ptr().add(x)
        }
    }
    unsafe fn cur_index(&self, buf: *mut VGABuf) -> *mut u16 {
        Self::index(
            buf,
            self.curr_column.load(Ordering::Relaxed),
            self.curr_row.load(Ordering::Relaxed)
        )
    }
    fn write_char(&self, c: char) {
        let buf = self.obtain();
        let mut c = match c {
            '\n' => {
                self.new_line(buf);
                self.release(buf);
                return;
            }
            c if c.is_ascii() && !c.is_ascii_control() => {
                c as u16
            }
            _ => {
                219u16
            }
        };
        // TODO(Bryce): Allow for other colors somehow?
        c |= 0x0f00; // Black background, white foreground
        let mut x = self.curr_column.fetch_add(1, Ordering::Acquire);
        let mut y = self.curr_row.load(Ordering::Acquire);
        if x >= VGA_COLS {
            x -= VGA_COLS;
            y = self.curr_row.fetch_add(1, Ordering::Acquire) + 1;
            if y >= VGA_ROWS {
                x -= VGA_ROWS;
                self.curr_row.fetch_sub(VGA_ROWS, Ordering::Release);
            }
            self.curr_column.fetch_sub(VGA_COLS, Ordering::Release);
        }
        // IMPORTANT(bryce): Safe for ... reasons
        unsafe {
            Self::index(buf, x, y).write_volatile(c);
        }
        self.release(buf);
    }
    fn new_line(&self, buf: *mut VGABuf)
    {
        let rem = VGA_COLS - self.curr_column.load(Ordering::Acquire);
        // IMPORTANT(bryce): Yea ... this is probably not sound
        unsafe {
            volatile_set_memory(self.cur_index(buf), 0, rem);
        }
        self.curr_column.store(0, Ordering::Release);
        self.curr_row.fetch_add(1, Ordering::Acquire);
    }
}

impl Write for VGAWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        s.chars().for_each(|c| self.0.write_char(c));
        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result
    {
        self.0.write_char(c);
        Ok(())
    }
}

pub fn test_panic() -> ! {
    panic!("I am panicking from inside vga/mod.rs!!! Oh no!");
}
