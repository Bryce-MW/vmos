#! /bin/bash

kernel=build/bootloader

#   -d int \

#qemu-system-x86_64 \
#  "$kernel" \
#  -s \
#  -monitor stdio

#qemu-system-x86_64 \
#  "$kernel" \
#  -s \
#  -S \
#  -no-reboot \
#  -monitor stdio

#qemu-system-x86_64 \
#  -machine q35,accel=tcg,vmport=off \
#  -boot menu=on \
#  -cpu max \
#  -smp cpus=6 \
#  -name vmos \
#  -drive file="$kernel",media=disk,if=virtio,format=raw \
#  -vga virtio \
#  -monitor stdio

qemu-system-x86_64 \
  -machine q35,accel=tcg,vmport=off \
  -s \
  -cpu max \
  -smp cpus=6 \
  -name vmos \
  -device ioh3420,id=root_port1 \
  -drive file="$kernel",media=disk,if=none,format=raw,id=disk \
  -device virtio-blk-pci,bus=root_port1,drive=disk,cyls=1,heads=1,secs=1 \
  -vga virtio \
  -chardev vc,id=seabios -device isa-debugcon,iobase=0x402,chardev=seabios \
  -monitor stdio \
#  -boot menu=on \
#  -S
