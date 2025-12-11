!#/bin/bash
set -ev

RUSTFLAGS="-C link-arg=-Tkernel.ld -C linker=rust-lld" \
  cargo build --bin kernel --target riscv64gc-unknown-none-elf

cp kernel/target/riscv64gc-unknown-none-elf/debug/kernel kernel.elf

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
    -kernel kernel.elf
