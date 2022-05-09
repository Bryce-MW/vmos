set print asm-demangle on
set disassembly-flavor intel

symbol-file /Users/bryce/randomProj/vmos/build/kernel -o 0x18000
add-symbol-file /Users/bryce/randomProj/vmos/build/bootloader.elf -o 0x7c00
set substitute-path /root/vmos /Users/bryce/randomProj/vmos
set substitute-path /jai /Users/bryce/randomProj/jai

break kernel.jai:6
break write
