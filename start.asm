%use ifunc
bits 16

; ES:DI = Pointer to PnP Installation Check Structure
; DL = Drive number used for the INT 13h (00h, 80h, etc.)

; 0x1000 will be the location of the first page table
PML4  equ 0x1000
; 0x2000 will be the location of the second page table
PDPT  equ 0x2000
PageTableEntries equ 512

MemoryMap equ 0x3000

KernelLoadStart equ 0x8000
KernelLoadMax   equ KernelLoadStart + 0x10000 ; Size of a segment
KernelLocation  equ KernelLoadMax
%if KERNEL_LENGTH % 512
    KernelSectors      equ (KERNEL_LENGTH / 512) + 1
%else
    KernelSectors      equ KERNEL_LENGTH / 512
%endif

GNURelRoNumber equ 0x6474e552

struc ElfHeader
    .ident:     resb 16
    .type:      resw 1
    .machine:   resw 1
    .version:   resd 1
    .entry:     resq 1
    .phoff:     resq 1
    .shoff:     resq 1
    .flags:     resd 1
    .ehsize:    resw 1
    .phentsize: resw 1
    .phnum:     resw 1
    .shentsize: resw 1
    .shnum:     resw 1
    .shstrndx:  resw 1
endstruc
struc ElfSectionHeader
    .name:     resd 1
    .type:     resd 1
    .flags:    resq 1
    .addr:     resq 1
    .offset:   resq 1
    .size:     resq 1
    .link:     resd 1
    .info:     resd 1
    .addralign resq 1
    .entsize   resq 1
endstruc
struc ElfRela
    .offset resq 1
    .info   resq 1
    .addend resq 1
endstruc

section .text
%if ELF == 0
org 0x7c00
%endif
jmp 0x0:_start; Far jump so cs is properly cleared
_start:
; Clean up segment registers and add stack below us just in case
xor ax, ax
mov ss, ax
mov sp, _start
mov ds, ax
mov es, ax
mov fs, ax
mov gs, ax
; Store boot drive number (we may be able to eliminate this?)
mov [DriveNumber], dl
; Disable A20 gate (do we need this?)
in al, 0x92
or al, 2
out 0x92, al
; Tell BIOS that we will eventually be using long mode only. (optional)
mov ax, 0xEC00
mov bl, 2
int 0x15

; Get the memory map and store that somewhere
mov di, MemoryMap
xor ebx, ebx
mov edx, 0x534D4150
NextMemoryMap:
mov eax, 0xE820
mov ecx, 24
int 0x15
jc DoneMemoryMap
add di, 24
test ebx, ebx
jnz NextMemoryMap
DoneMemoryMap:
mov dword [di], 0xFFFFFFFF
mov dword [di + 4], 0xFFFFFFFF

; TODO(bryce): Deal with larger kernels in reading
; Load the kernel! Max size supported is 64k for now...
mov si, DiskAddressPacket
mov ah, 0x42
mov dl, [DriveNumber]
int 0x13
; Elf header is now at 0x8000

; 1) Build paging structures (PML4, PDPT, PD and PTs)
; Fill in an entry for the PDPT
mov dword [PML4], 0b11 | (2 << 12)
; Clear out the rest of this entry and the other entries
mov di, PML4 + 4
mov cx, (PageTableEntries) * 8 - 4
xor al, al
cld
rep stosb
; Create the PDPT to identity map 1GB of memory
mov dword [PDPT], 0b10000011
; Clear out the rest of this entry and the other entries
mov di, PDPT + 4
mov cx, (PageTableEntries) * 8 - 4
rep stosb

; Disable interrupts
mov al, 0xFF
out 0xA1, al
out 0x21, al
; Make NMI's crash (do we need this for a VM?)
lidt [idt]

; 2) Enable PAE in CR4
mov eax, 10100000b
mov cr4, eax

; 3) Set CR3 so it points to the PML4
mov eax, PML4
mov cr3, eax

; 4) Enable long mode in the EFER MSR
mov ecx, 0xc0000080
rdmsr
bts ax, 8 ; Add 1 in the future to enable syscall instruction
wrmsr

; 5) Enable paging and protected mode at the same time (activate long mode)
;        paging      protected mode
mov eax, (1 << 31) | 1
mov cr0, eax
; 6) Load a GDT
lgdt [gdt.pointer]
; 7) Do a "far jump" to some 64 bit code
jmp 0x8:_start64


_start64:
BITS 64
; Set everyone but cs to the data segment (some could theoretically be null or used for TLS)
mov ax, 0x10
mov ds, ax
mov es, ax
mov fs, ax
mov gs, ax
mov ss, ax

; Parse the kernel elf file data
movzx rcx, word [KernelLoadStart + ElfHeader.shnum]
push rcx
mov rbx, KernelLoadStart
add rbx, [KernelLoadStart + ElfHeader.shoff]
push rbx
mov rax, [KernelLoadStart + ElfHeader.entry]
add [KernelStart], rax
parseData:
    cmp qword [rbx + ElfSectionHeader.addr], 0
    je .skipParseElf
        push rcx
        mov rcx, [rbx + ElfSectionHeader.size]
        mov rsi, KernelLoadStart
        add rsi, [rbx + ElfSectionHeader.offset]
        mov rdi, KernelLocation
        add rdi, [rbx + ElfSectionHeader.addr]
        cmp dword [rbx + ElfSectionHeader.type], 8
        je .zero
            rep movsb
        jmp .skipStoZero
        .zero:
            xor eax, eax
            rep stosb
        .skipStoZero:
        pop rcx
    .skipParseElf:
    add rbx, ElfSectionHeader_size
loop parseData
; Parse the kernel elf relocations
pop rbx
pop rcx
parseRela:
    cmp dword [rbx + ElfSectionHeader.type], 4
    jne .notRela
        mov rsi, [rbx + ElfSectionHeader.offset]
        add rsi, KernelLoadStart
        mov rdx, [rbx + ElfSectionHeader.size]
        .parseRela:
            cmp rdx, 0
            je .notRela
            sub rdx, ElfRela_size
            mov rax, [rsi + rdx + ElfRela.addend]
            add rax, KernelLocation
            mov rdi, KernelLocation
            add rdi, [rsi + rdx + ElfRela.offset]
            add [rdi], rax
        jmp .parseRela
    .notRela:
    add rbx, ElfSectionHeader_size
loop parseRela

kLoop:
call [KernelStart]
jmp kLoop ; If the kernel returns, just call it again lol


section .data
align 4
DiskAddressPacket:
.size:     db 16
.reserved: db 0
.sectors:  dw KernelSectors
.buf.off:  dw 0
.buf.seg:  dw KernelLoadStart >> 4
.lba.l:    dq 1
.lba.h:    dd 0

; idt is shorter than a quad word and is all null for now we can hide it in the gdt
idt:
    ;.length: dw 0
    ;.base:   dd 0

gdt:
.null:
    dq 0x0 ; Unused null descriptor

.code:
    dq 0x00209A0000000000 ; code segment (exec, read)
    dq 0x0000920000000000 ; data segment (read, rite)

.pointer:
    dw $ - gdt - 1 ; gdt size
    dd gdt         ; gdt address

KernelStart:
    dq KernelLocation
;    dq 0 ; Setting this to 0 since we are making the image locations absolute

RelaLoc:
    dq 0
RelaLen:
    dq 0

DriveNumber:
    db 0

%if ELF == 0
section .mbr start=510+0x7c00
db 0x55, 0xAA
%endif
