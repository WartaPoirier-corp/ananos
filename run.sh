#!/usr/bin/env bash

set -e

d=$(pwd)
mkdir -p $d/target/kernel
cargo build

pushd ~/.cargo/registry/src/*/bootloader-0.10.1/

cargo builder --kernel-manifest $d/Cargo.toml --out-dir $d/target/kernel --target-dir $d/target --kernel-binary $d/target/x86_64-os/debug/os

popd

qemu-system-x86_64 -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -drive format=raw,file=target/kernel/boot-bios-os.img --no-shutdown
