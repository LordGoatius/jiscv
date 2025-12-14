#![no_std]
#![no_main]

use core::{arch::naked_asm, panic::PanicInfo};

use shell::{exit, println, Shell};

unsafe extern "C" {
    static mut __stack_top: u8;
}

#[unsafe(no_mangle)]
fn main() {
    println!("Hello! You are now entering the shell");
    Shell::default().enter();
    // exit();
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
