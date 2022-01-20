# VMOS (name pending)

This OS is meant for single-application VMs. In many cases, Docker will be your best bet and is a lot easier to develop
for but use this if you really need the max performance (numbers pending) or you want the low overhead without having
control of the hypervisor (on public cloud for example). Right now, I am only planning on writing VirtIO drivers which
will make using this on a public cloud more difficult. I may add them in the future but feel free to contribute them if
you want (but why would you this early in the project?). I haven't added the license file yet, but it's going to be the
BSD-3-Clause. The rest of this readme is just my own notes, so I remember what I did and what I want to do.

## Boot Process
* Starts in start.asm
* Store the memory map for later
* Load kernel into low memory
* Get into long mode asap
* Load the kernel and do relocations (kernel currently loaded into low memory but eventually this will likely change)
* Jump into the kernel
* Kernel reads the memory map and creates new paging structures to create a flat section of usable memory for the kernel
  to use
* Reload the kernel into this new memory
* Boot into the kernel, but we are now in high half mode

## Memory map
### Initial
 * `0x1000 - 0x2000`: PML4 (The highest level page table)
 * `0x2000 - 0x3000`: PDPT (The second level page table)
 * `0x3000 - 0x????`: Memory map
 * `0x???? - 0x7c00`: Stack
 * `0x8000 - 0x18000`: Position of kernel file
 * `0x18000 - 0x?????`: Position kernel is loaded to
### High half mode
 * `0xFFFF800000000000 - 0xFFFFC00000000000`: Scratch area for MMIO etc
 * `0xFFFFC00000000000 - 0xFFFFFF0000000000`: Linear usable memory
 * `0xFFFFFF0000000000 - 0x10000000000000000`: 1024 1GB stacks
 * `0xFFFFFF0000000000 - 0xFFFFFF0040000000`: Initial stack
### Mapping process
 * Use memory map to split memory up into maximal 1G, 2M, and 4k pages.
 * Subtract important boot pages
 * Allocate one 2M page for initial stack (split up 1G page if needed)
 * Put the rest of the pages into the linear usable memory space

This is not a perfect method because it does not allow for a more allocations out of the linear usable memory. What I
would want to do in the future is to write a better memory allocator that keeps a pool of free pages and only allocates
what is needed. But this will be a lot simpler and will work for now
