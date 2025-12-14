#!/bin/bash
set -ev

cargo build --bin shell --target riscv64gc-unknown-none-elf

cp target/riscv64gc-unknown-none-elf/debug/shell .shell.elf

llvm-objcopy --set-section-flags .bss=alloc,contents -O binary .shell.elf .shell.bin
llvm-objcopy -Ibinary -Oelf64-littleriscv .shell.bin .shell.bin.o
./flags.py

RUSTFLAGS="-C link-arg=.shell.bin.o" \
    cargo build --bin kernel --target riscv64gc-unknown-none-elf

cp target/riscv64gc-unknown-none-elf/debug/kernel kernel.elf

qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -bios default \
    -smp 1 \
    -m 128M \
    -nographic \
    -d cpu_reset,unimp,guest_errors,int -D qemu.log \
    -serial mon:stdio \
    --no-reboot \
    -drive id=drive0,file=lorem.txt,format=raw,if=none \
    -device virtio-blk-device,drive=drive0,bus=virtio-mmio-bus.0 \
    -kernel kernel.elf
