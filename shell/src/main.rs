#![no_std]
#![no_main]

use core::{arch::naked_asm, panic::PanicInfo};

use shell::putchar;

unsafe extern "C" {
    static mut __stack_top: u8;
}


#[unsafe(no_mangle)]
fn main() {
    // enter user mode
    putchar(b'[');
    putchar(b'F');
    putchar(b'R');
    putchar(b'O');
    putchar(b'M');
    putchar(b'U');
    putchar(b'S');
    putchar(b'E');
    putchar(b'R');
    putchar(b'S');
    putchar(b'P');
    putchar(b'A');
    putchar(b'C');
    putchar(b'E');
    putchar(b']');

    putchar(b':');
    putchar(b' ');

    putchar(b'H');
    putchar(b'e');
    putchar(b'l');
    putchar(b'l');
    putchar(b'o');
    putchar(b' ');
    putchar(b'w');
    putchar(b'o');
    putchar(b'r');
    putchar(b'l');
    putchar(b'd');
    putchar(b'\n');
    loop {}
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn start() {
    naked_asm!(
        "la sp, __stack_top",
        "call {main}",
        "call exit",
        main = sym main
    )
}


#[panic_handler]
pub fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
