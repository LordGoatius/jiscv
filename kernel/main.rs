#![feature(
    arbitrary_self_types,
    arbitrary_self_types_pointers,
    ascii_char,
    ascii_char_variants,
    const_default,
    const_trait_impl,
    custom_inner_attributes,
    debug_closure_helpers,
    int_from_ascii,
    generic_const_exprs, // I accept the risk, but it's so cool and useful
    naked_functions_rustic_abi,
    pointer_is_aligned_to,
    ptr_as_ref_unchecked,
    riscv_ext_intrinsics,
    str_from_raw_parts,
    trim_prefix_suffix,
)]
#![allow(static_mut_refs)]
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
mod uart;
mod virtio;
mod tar;
mod ext2;
mod syscall;
pub mod traits;

mod dtree;
mod user;

use core::arch::asm;
use core::panic::PanicInfo;

use ralloc::vec::Vec;
use spin::lazy::Lazy;

use crate::alloc::GLOBAL_ALLOC;
use crate::dtree::{DeviceTreeHeader, DeviceTreeNode};
use crate::proc::{create_process, r#yield, Process};
use crate::tar::File;
use crate::uart::{UartInitError, init_uart};
use crate::user::{_binary__shell_bin_end, _binary__shell_bin_start};

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

// Use trait for VFS later lol
pub trait Filesystem {}
pub static mut FILESYSTEM: Option<Vec<File>> = None;

// Don't really think this is safe
pub fn get_fs_unwrap() -> &'static mut [File] {
    unsafe {
        FILESYSTEM.as_mut().unwrap()
    }
}

fn main() -> ! {
    let (devicetree, hart_start) = unsafe {
        let temp1: usize;
        let temp2: usize;
        asm!(
            "mv {}, a1",
            "mv {}, a0",
            out(reg) temp1,
            out(reg) temp2,
        );
        (temp1 as *mut DeviceTreeHeader, temp2)
    };

    unsafe {
        let bss_start = &raw mut __bss;
        let bss_size = (&raw mut __bss_end as usize) - (&raw mut __bss as usize);
        core::ptr::write_bytes(bss_start, 0, bss_size);

        asm!("csrw stvec, {}", in(reg) trap::trap_entry as *const u8);
    }

    println!("Booting JimOS");
    println!("Starting Hart: {hart_start}");

    GLOBAL_ALLOC.init(&raw mut __heap, &raw mut __heap_end);

    let dtree = dtree::parse(devicetree);
    // dtree.print_properties();
    let node = dtree.search("/soc/serial");
    let addr = node.and_then(DeviceTreeNode::get_addr);
    let addr = addr.and_then(
        |src| usize::from_str_radix(src, 16).ok().map(|num| num as *mut u8)
    );
    let addr = addr
        .ok_or_else(|| UartInitError)
        .and_then(|addr| init_uart(addr));

    virtio::init_virtio();

    interrupt::interrupt_enable();

    ext2::init();

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
