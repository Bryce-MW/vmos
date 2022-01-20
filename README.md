# VMOS (name pending)

This OS is meant for single-application VMs. In many cases, Docker will be your best bet and is a lot easier to develop
for but use this if you really need the max performance (numbers pending) or you want the low overhead without
having control of the hypervisor (on public cloud for example). Right now, I am only planning on writing VirtIO drivers
which will make using this on a public cloud more difficult. I may add them in the future but feel free to contribute
them if you want (but why would you this early in the project?). I haven't added the license file yet, but it's going to
be the BSD-3-Clause. The rest of this readme is just my own notes, so I remember what I did and what I want to do.

# Boot Process
* Starts in start.asm
* Store the memory map for later
* Load kernel into low memory
* Get into long mode asap
* Load the kernel and do relocations (kernel currently loaded into low memory but eventually this will likely change)
* Jump into the kernel
* Kernel reads the memory map and creates new paging structures to create a flat section of usable memory for the kernel to use 
* Reload the kernel into this new memory
* Boot into the kernel, but we are now in high half mode
