#!/usr/bin/env bash

set -e

cargo kbuild
cargo boot
qemu-system-x86_64 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -drive format=raw,file=target/x86_64-os/debug/boot-bios-os.img --no-shutdown -d int
