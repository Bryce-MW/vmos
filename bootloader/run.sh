#! /bin/bash

kernel=build/bootloader

#   -d int \

#qemu-system-x86_64 \
#  "$kernel" \
#  -s \
#  -monitor stdio

qemu-system-x86_64 \
  "$kernel" \
  -s \
  -S \
  -no-reboot \
  -monitor stdio
