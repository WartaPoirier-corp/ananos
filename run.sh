#!/usr/bin/env bash

set -e

clang -target x86_64-none-none -c -o test.o test.s
objcopy -I elf64-little -j .text -O binary test.o test.bin
cargo kbuild
cargo boot
qemu-system-x86_64 -machine q35 -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -device qemu-xhci,id=xhci,bus=pcie.0 \
    -device usb-mouse,bus=xhci.0 \
    -serial stdio \
    -drive format=raw,file=target/x86_64-os/debug/boot-bios-os.img \
    --no-shutdown
