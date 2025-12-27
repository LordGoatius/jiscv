use core::arch::asm;

use crate::interrupt::SSTATUS_SIE;

pub const USER_BASE: usize = 0x1000000;

pub const SSTATUS_SPIE: usize = 1 << 5;
pub const SSTATUS_SUM: usize = 1 << 18;

unsafe extern "C" {
    pub static mut _binary__shell_bin_start: u8;
    pub static mut _binary__shell_bin_end: u8;
}

pub fn userspace_entry() {
    write_csr!("sepc", USER_BASE);
    write_csr!("sstatus", SSTATUS_SPIE | SSTATUS_SIE | SSTATUS_SUM);
    unsafe { asm!("sret") }
}
