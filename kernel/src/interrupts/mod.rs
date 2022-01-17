use core::{
    marker::PhantomData,
    mem::{size_of_val, MaybeUninit},
    ops::Deref
};

#[allow(unused)]
pub fn sti() { unsafe { asm!("sti", options(nomem, preserves_flags, nostack)) } }

#[allow(unused)]
pub fn cti() { unsafe { asm!("cti", options(nomem, preserves_flags, nostack)) } }

#[repr(C, packed)]
struct IDTR
{
    limit: u16,
    base:  *mut IDT
}

#[repr(C)]
struct IDT
{
    de:  IDTEnt<IntFunc>,
    db:  IDTEnt<IntFunc>,
    nmi: IDTEnt<IntFunc>,
    bp:  IDTEnt<IntFunc>,
    of:  IDTEnt<IntFunc>,
    br:  IDTEnt<IntFunc>,
    ud:  IDTEnt<IntFunc>,
    nm:  IDTEnt<IntFunc>,
    df:  IDTEnt<BErFunc>,
    co:  IDTEnt<IntFunc>,
    ts:  IDTEnt<ErrFunc>,
    np:  IDTEnt<ErrFunc>,
    ss:  IDTEnt<ErrFunc>,
    gp:  IDTEnt<ErrFunc>,
    pf:  IDTEnt<ErrFunc>,
    _a:  IDTEnt<IntFunc>,
    mf:  IDTEnt<IntFunc>,
    ac:  IDTEnt<ErrFunc>,
    mc:  IDTEnt<BadFunc>,
    xm:  IDTEnt<IntFunc>,
    ve:  IDTEnt<IntFunc>,
    cp:  IDTEnt<ErrFunc>
}

static mut GLOB_IDT: MaybeUninit<IDT> = MaybeUninit::uninit();

// IMPORTANT(bryce): This is very unsafe. Please make sure to only run this once
pub unsafe fn create_glob_idt()
{
    let idt = GLOB_IDT.write(IDT::new());
    let idtr = IDTR {
        limit: size_of_val(idt) as u16,
        base:  idt
    };
    asm!("lidt [{}]", in(reg) &idtr, options(readonly, preserves_flags, nostack));
}

impl IDT
{
    fn new() -> Self
    {
        IDT {
            de:  IDTEnt::<IntFunc>::new(panic),
            db:  IDTEnt::<IntFunc>::new(panic),
            nmi: IDTEnt::<IntFunc>::new(panic),
            bp:  IDTEnt::<IntFunc>::new(panic),
            of:  IDTEnt::<IntFunc>::new(panic),
            br:  IDTEnt::<IntFunc>::new(panic),
            ud:  IDTEnt::<IntFunc>::new(panic),
            nm:  IDTEnt::<IntFunc>::new(panic),
            df:  IDTEnt::<BErFunc>::new(panic_ber),
            co:  IDTEnt::<IntFunc>::new(panic),
            ts:  IDTEnt::<ErrFunc>::new(panic_err),
            np:  IDTEnt::<ErrFunc>::new(panic_err),
            ss:  IDTEnt::<ErrFunc>::new(panic_err),
            gp:  IDTEnt::<ErrFunc>::new(panic_err),
            pf:  IDTEnt::<ErrFunc>::new(panic_err),
            _a:  IDTEnt::<IntFunc>::new_empty(),
            mf:  IDTEnt::<IntFunc>::new(panic),
            ac:  IDTEnt::<ErrFunc>::new(panic_err),
            mc:  IDTEnt::<BadFunc>::new(panic_bad),
            xm:  IDTEnt::<IntFunc>::new(panic),
            ve:  IDTEnt::<IntFunc>::new(panic),
            cp:  IDTEnt::<ErrFunc>::new(panic_err)
        }
    }
}

#[repr(C, packed)]
struct IDTEnt<T>
{
    offset_low:        u16,
    code_elector:      u16,
    stack_table_index: u8,
    flags:             u8,
    offset_high:       u16,
    offset_extended:   u32,
    reserved:          u32,
    handler:           PhantomData<T>
}

impl<T> IDTEnt<T>
{
    // Interrupt gate is 0b1110
    // Trap gate is 0b1111
    // Call gate is 0b1100
    fn new_empty() -> Self
    {
        IDTEnt {
            offset_low:        0,
            code_elector:      0x8,
            stack_table_index: 0,
            flags:             0b1110,
            offset_high:       0,
            offset_extended:   0,
            reserved:          0,
            handler:           PhantomData
        }
    }
    unsafe fn new_unchecked(offset: u64) -> Self
    {
        let mut res = Self::new_empty();
        res.offset_low = (offset & 0xffff) as u16;
        res.offset_high = ((offset & 0xffff0000) >> 16) as u16;
        res.offset_extended = (offset >> 32) as u32;
        res.set_present();
        res
    }
    fn set_present(&mut self) { self.flags |= 1 << 7; }
}

type IntFunc = extern "x86-interrupt" fn(InterruptFrame);
type ErrFunc = extern "x86-interrupt" fn(InterruptFrame, error: u64);
type BadFunc = extern "x86-interrupt" fn(InterruptFrame) -> !;
type BErFunc = extern "x86-interrupt" fn(InterruptFrame, error: u64) -> !;

impl IDTEnt<IntFunc>
{
    fn new(address: IntFunc) -> Self { unsafe { Self::new_unchecked(address as u64) } }
}
impl IDTEnt<ErrFunc>
{
    fn new(address: ErrFunc) -> Self { unsafe { Self::new_unchecked(address as u64) } }
}
impl IDTEnt<BadFunc>
{
    fn new(address: BadFunc) -> Self { unsafe { Self::new_unchecked(address as u64) } }
}
impl IDTEnt<BErFunc>
{
    fn new(address: BErFunc) -> Self { unsafe { Self::new_unchecked(address as u64) } }
}

// NOTE(bryce): Wrapper is used for safety
#[repr(C)]
struct InterruptFrame(InterruptFrameInternal);
impl Deref for InterruptFrame
{
    type Target = InterruptFrameInternal;
    fn deref(&self) -> &Self::Target { &self.0 }
}
#[repr(C)]
pub struct InterruptFrameInternal
{
    pub instruction_pointer: *mut u8,
    pub code_segment:        u64,
    pub cpu_flags:           u64,
    pub stack_pointer:       *mut u8,
    pub stack_segment:       u64
}

#[allow(unused)]
mod page_errors
{
    const PROTECTION_VIOLATION: u64 = 1;
    const CAUSED_BY_WRITE: u64 = 1 << 1;
    const USER_MODE: u64 = 1 << 2;
    const MALFORMED_TABLE: u64 = 1 << 3;
    const INSTRUCTION_FETCH: u64 = 1 << 4;
    const PROTECTION_KEY: u64 = 1 << 5;
    const SHADOW_STACK: u64 = 1 << 6;
    const SGX: u64 = 1 << 15;
    const RMP: u64 = 1 << 31;
}

extern "x86-interrupt" fn panic(frame: InterruptFrame)
{
    panic!("Unexpected interrupt at: {:p}", frame.instruction_pointer);
}
extern "x86-interrupt" fn panic_err(frame: InterruptFrame, error: u64)
{
    panic!(
        "Unexpected interrupt with err: {:x} at: {:p}",
        error, frame.instruction_pointer
    );
}
extern "x86-interrupt" fn panic_bad(frame: InterruptFrame) -> !
{
    panic!("Unexpected interrupt at: {:p}", frame.instruction_pointer)
}
extern "x86-interrupt" fn panic_ber(frame: InterruptFrame, error: u64) -> !
{
    panic!(
        "Unexpected interrupt with err: {:x} at: {:p}",
        error, frame.instruction_pointer
    )
}
