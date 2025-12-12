use core::arch::asm;

pub const USER_BASE: usize = 0x1000000;

const SSTATUS_SPIE: usize = 1 << 5;

unsafe extern "C" {
    pub static mut _binary__shell_bin_start: u8;
    pub static mut _binary__shell_bin_end: u8;
}

pub fn userspace_entry() {
    write_csr!("sepc", USER_BASE);
    write_csr!("sstatus", SSTATUS_SPIE);
    unsafe { asm!("sret") }
}
