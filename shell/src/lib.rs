#![no_std]

use crate::syscall::{SYS_PUTCHAR, syscall};

mod syscall;

#[unsafe(no_mangle)]
pub fn exit() -> ! {
    loop {}
}

pub fn putchar(char: u8) {
    syscall(SYS_PUTCHAR, char as usize, 0, 0);
}
