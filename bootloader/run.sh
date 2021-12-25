#! /bin/bash

#   -d int \

#qemu-system-x86_64 \
#  build/bootloader \
#  -s \
#  -monitor stdio

qemu-system-x86_64 \
  build/bootloader \
  -s \
  -S \
  -no-reboot \
  -monitor stdio
