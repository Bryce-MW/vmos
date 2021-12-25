#! /bin/bash

set -ex

mkdir build 2>/dev/null || true

kernel=../target/x86_64-unknown-none/release/kernel

/usr/local/opt/binutils/bin/strip -s -o "$kernel"-s "$kernel"
k_size="$(stat -f '%z' "$kernel"-s)"

nasm -f bin -g -w+orphan-labels -dKERNEL_LENGTH="$k_size" -dELF=0 -o build/bootloader-head src/start.asm
nasm -f elf -g -F dwarf -w+orphan-labels -dKERNEL_LENGTH="$k_size" -dELF=1 -o build/bootloader-head.elf src/start.asm

cat build/bootloader-head "$kernel"-s > build/bootloader
