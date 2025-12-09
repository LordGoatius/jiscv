#![feature(naked_functions_rustic_abi, const_trait_impl, const_default, pointer_is_aligned_to)]
#![no_std]
#![no_main]

extern crate alloc as ralloc;

mod alloc;
mod proc;
mod sbi;
mod trap;
mod paging;

use core::arch::asm;
use core::panic::PanicInfo;

use spin::lazy::Lazy;

use crate::alloc::GLOBAL_ALLOC;
use crate::proc::{Process, create_process, r#yield};

unsafe extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
    static mut __heap: u8;
    static mut __heap_end: u8;
    static mut __kernel_base: u8;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn boot() -> ! {
    unsafe {
        asm!(
            "la sp, __stack_top",
            "j {main}",
            main = sym main,
            options(noreturn)
        );
    }
}

static mut PROC_IDLE: Lazy<*mut Process> = Lazy::new(|| create_process(core::ptr::null_mut() as *const () as usize).unwrap());
static mut PROC_A: Lazy<*mut Process> = Lazy::new(|| create_process(proc_a_entry as *const () as usize).unwrap());
static mut PROC_B: Lazy<*mut Process> = Lazy::new(|| create_process(proc_b_entry as *const () as usize).unwrap());

pub static mut PROC_CURR: Option<*mut Process> = None;

fn proc_a_entry() {
    println!("Entering Proc A:");
    loop {
        println!("A");
        r#yield();
    }
}

fn proc_b_entry() {
    println!("Entering Proc B:");
    loop {
        println!("B");
        r#yield();
    }
}

fn main() -> ! {
    unsafe {
        let bss_start = &raw mut __bss;
        let bss_size = (&raw mut __bss_end as usize) - (&raw mut __bss as usize);
        core::ptr::write_bytes(bss_start, 0, bss_size);

        asm!("csrw stvec, {}", in(reg) trap::trap_entry as *const u8);
    }

    GLOBAL_ALLOC.init(&raw mut __heap, &raw mut __heap_end);

    {
        let idle;
        let proc_a;
        let proc_b;

        unsafe {
            idle = *PROC_IDLE;
            PROC_CURR = Some(idle);
            let _ = idle.read_volatile();

            proc_b = *PROC_B;
            proc_a = *PROC_A;

            let _ = proc_a.read_volatile().pid;
            let _ = proc_b.read_volatile().pid;
        }
    }

    println!("Booting JimOS");

    r#yield();

    loop {}
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("PanicInfo: {info}");
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
