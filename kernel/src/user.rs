use core::arch::asm;

use crate::write_csr;

pub const USER_BASE: usize = 0x1000000;

const SSTATUS_SPIE: usize = 1 << 5;

unsafe extern "C" {
    pub static mut _binary__shell_bin_start: u8;
    pub static mut _binary__shell_bin_end: u8;
}

pub fn userspace_entry() {
    write_csr!("sepc", USER_BASE);
    write_csr!("sstatus", SSTATUS_SPIE);
    unsafe {
        asm!("sret")
    }
    // naked_asm!(
    //     "csrw sepc, {sepc}",
    //     "csrw sstatus, {sstatus}",
    //     "sret",
    //     sepc = const USER_BASE,
    //     sstatus = const SSTATUS_SPIE
    // )
}

