use core::{
    fmt::{self, Write},
    intrinsics::volatile_set_memory,
    ptr,
    sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize, Ordering}
};

const VGA_COLS: usize = 80;
const VGA_ROWS: usize = 25;
type VGABuf = [[u16; VGA_COLS]; VGA_ROWS];
pub static VGA: VGA = VGA {
    ptr:         AtomicPtr::new(0xb8000 as *mut VGABuf),
    curr_row:    AtomicUsize::new(0),
    curr_column: AtomicUsize::new(0),
    color:       AtomicU8::new(0x0f)
};
pub struct VGA
{
    ptr:         AtomicPtr<VGABuf>, /* This really doesn't need to be atomic but this whole
                                     * abstraction is bad */
    curr_row:    AtomicUsize,
    curr_column: AtomicUsize,
    pub color:   AtomicU8
}

pub struct VGAWriter<'a>(&'a VGA);

impl VGA
{
    pub fn writer(&self) -> VGAWriter { VGAWriter(&self) }
    unsafe fn clear(&self, buf: *mut VGABuf)
    {
        // Clear the screen
        volatile_set_memory(buf, 0, 1);
        self.curr_row.store(0, Ordering::Release);
        self.curr_column.store(0, Ordering::Release);
    }
    fn obtain(&self) -> *mut VGABuf
    {
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
    fn release(&self, buf: *mut VGABuf) { self.ptr.store(buf, Ordering::Release); }
    unsafe fn index(buf: *mut VGABuf, x: usize, y: usize) -> *mut u16
    {
        (*buf)[y].as_mut_ptr().add(x)
    }
    unsafe fn cur_index(&self, buf: *mut VGABuf) -> *mut u16
    {
        Self::index(
            buf,
            self.curr_column.load(Ordering::Relaxed),
            self.curr_row.load(Ordering::Relaxed)
        )
    }
    fn write_char(&self, c: char)
    {
        let buf = self.obtain();
        let mut c = match c {
            '\n' => {
                self.new_line(buf);
                self.release(buf);
                return;
            },
            c if c.is_ascii() && !c.is_ascii_control() => c as u16,
            _ => 168u16
        };
        c |= (self.color.load(Ordering::Relaxed) as u16) << 8;
        let mut x = self.curr_column.fetch_add(1, Ordering::Acquire);
        let mut y = self.curr_row.load(Ordering::Acquire);
        if x >= VGA_COLS {
            x -= VGA_COLS;
            y = self.curr_row.fetch_add(1, Ordering::Acquire) + 1;
            self.curr_column.fetch_sub(VGA_COLS, Ordering::Release);
        }
        if y >= VGA_ROWS {
            y -= VGA_ROWS;
            self.curr_row.fetch_sub(VGA_ROWS, Ordering::Release);
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

impl Write for VGAWriter<'_>
{
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
