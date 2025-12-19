#![feature(
    arbitrary_self_types,
    arbitrary_self_types_pointers,
    ptr_as_ref_unchecked,
    naked_functions_rustic_abi,
    const_trait_impl,
    const_default,
    pointer_is_aligned_to,
    abi_riscv_interrupt
)]
#![no_std]
#![no_main]

extern crate alloc as ralloc;

mod alloc;
#[macro_use]
mod interrupt;
mod paging;
mod proc;
mod sbi;
#[macro_use]
mod print;
#[macro_use]
mod trap;
mod virtio;

mod dtree;
mod user;

use core::arch::asm;
use core::panic::PanicInfo;

use spin::lazy::Lazy;

use crate::alloc::GLOBAL_ALLOC;
use crate::dtree::DeviceTreeHeader;
use crate::proc::{create_process, r#yield, Process};
use crate::user::{_binary__shell_bin_end, _binary__shell_bin_start};
use crate::virtio::{SECTOR_SIZE, init_virtio, read_write_disk};

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

static mut PROC_IDLE: Lazy<*mut Process> =
    Lazy::new(|| create_process(core::ptr::null_mut(), 0).unwrap());

pub static mut PROC_CURR: Option<*mut Process> = None;

fn main() -> ! {
    let devicetree = unsafe {
        let temp: usize;
        asm!(
            "mv {}, a1",
            out(reg) temp,
        );
        temp as *mut DeviceTreeHeader
    };

    unsafe {
        let bss_start = &raw mut __bss;
        let bss_size = (&raw mut __bss_end as usize) - (&raw mut __bss as usize);
        core::ptr::write_bytes(bss_start, 0, bss_size);

        asm!("csrw stvec, {}", in(reg) trap::trap_entry as *const u8);
    }

    println!("Booting JimOS");

    GLOBAL_ALLOC.init(&raw mut __heap, &raw mut __heap_end);

    let dtree = dtree::parse(devicetree);
    println!("{:#?}", dtree);

    init_virtio();

    let mut buf: [u8; SECTOR_SIZE as usize] = [0; SECTOR_SIZE as usize];
    read_write_disk(buf.as_mut_ptr(), 0, false);

    println!("first sector: {}", unsafe { str::from_utf8_unchecked(&buf) });

    buf[0..22].copy_from_slice(b"Hello from the kernel!");
    read_write_disk(buf.as_mut_ptr(), 0, true);
    
    interrupt::interrupt_enable();

    unsafe {
        PROC_CURR = Some(*PROC_IDLE);
    }

    let _ = create_process(
        &raw mut _binary__shell_bin_start,
        &raw mut _binary__shell_bin_end as usize - &raw mut _binary__shell_bin_start as usize,
    );

    r#yield();

    // Entering kernel busy loop
    println!("Entering kernel wait period");
    loop {
        unsafe { asm!("wfi") }
    }
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
