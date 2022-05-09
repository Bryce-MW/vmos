InterruptFrame :: struct {
    instruction_pointer: *u8;
    code_segment:        u64;
    cpu_flags:           u64;
    stack_pointer:       *u8;
    stack_segment:       u64;
}

IDTEnt :: struct {
    offset_low:        u16 = 0;
    code_elector:      u16 = 0x8;
    stack_table_index: u8  = 0;
    flags:             u8  = 0b1110;
    offset_high:       u16 = 0;
    offset_extended:   u32 = 0;
    reserved:          u32 = 0;
}

filled_idt :: (handler: () #no_call) -> IDTEnt {
    using ent: IDTEnt = ---;
    defer return ent;

    offset := cast(u64)handler;

    offset_low        = cast,no_check(u16)offset;
    code_elector      = 0x8;
    stack_table_index = 0;
    flags             = 0b1000_1110;
    offset_high       = cast,no_check(u16)((offset & 0xffff0000) >> 16);
    offset_extended   = cast,no_check(u32)(offset >> 32);
    reserved          = 0;
}

IDTData :: struct {
    name: string;
    has_error_code: bool = false;
    empty: bool = false;
}

IDT_DATA :: IDTData.[
    .{"de"},
    .{"db"},
    .{"nmi"},
    .{"bp"},
    .{"of"},
    .{"br"},
    .{"ud"},
    .{"nm"},
    .{"df", true},
    .{"co"},
    .{"ts", true},
    .{"np", true},
    .{"ss", true},
    .{"gp", true},
    .{"pf", true},
    .{"_a", false, true},
    .{"mf"},
    .{"ac", true},
    .{"mc"},
    .{"xm"},
    .{"ve"},
    .{"cp", true}
];

#placeholder fill_idt;
#insert #run create_idt(IDT_DATA);

create_idt :: (data: [] IDTData) -> string {
    for data {

    }
}

IDTR :: struct {
    limit: u16;
    base: *IDT #align 2;
}

GLOB_IDT: IDT;

// Only run this function once
create_global_idt :: () {
    //fill_idt(*#placeholder);
    idtr := IDTR.{size_of(IDT), *GLOB_IDT};
    idtrp := *idtr;
    #asm {
        lidt [idtrp];
    }
}

/*
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

extern "x86-interrupt" fn panic_00(frame: InterruptFrame) {                  panic!("Unexpected interrupt de  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_01(frame: InterruptFrame) {                  panic!("Unexpected interrupt db  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_02(frame: InterruptFrame) {                  panic!("Unexpected interrupt nmi at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_03(frame: InterruptFrame) {                  panic!("Unexpected interrupt bp  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_04(frame: InterruptFrame) {                  panic!("Unexpected interrupt of  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_05(frame: InterruptFrame) {                  panic!("Unexpected interrupt br  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_06(frame: InterruptFrame) {                  panic!("Unexpected interrupt ud  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_07(frame: InterruptFrame) {                  panic!("Unexpected interrupt nm  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_08(frame: InterruptFrame, error: u64) -> ! { panic!("Unexpected interrupt df  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_09(frame: InterruptFrame){                   panic!("Unexpected interrupt co  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_10(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt ts  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_11(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt np  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_12(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt ss  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_13(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt gp  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_14(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt pf  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_16(frame: InterruptFrame){                   panic!("Unexpected interrupt mf  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_17(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt ac  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
extern "x86-interrupt" fn panic_18(frame: InterruptFrame) -> ! {             panic!("Unexpected interrupt mc  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_19(frame: InterruptFrame) {                  panic!("Unexpected interrupt xm  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_20(frame: InterruptFrame) {                  panic!("Unexpected interrupt ve  at: {:p}", frame.instruction_pointer) }
extern "x86-interrupt" fn panic_21(frame: InterruptFrame, error: u64) {      panic!("Unexpected interrupt cp  with err: {:x} at: {:p}", error, frame.instruction_pointer) }
*/
