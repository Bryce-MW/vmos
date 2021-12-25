symbol-file /Users/bryce/CLionProjects/vmos/target/x86_64-unknown-none/release/kernel -o 0x18000
add-symbol-file /Users/bryce/CLionProjects/vmos/bootloader/build/bootloader-head.elf -o 0x7c00
break _start
