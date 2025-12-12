use core::arch::asm;

pub const SYS_PUTCHAR: usize = 1;
pub const SYS_GETCHAR: usize = 2;
pub const SYS_EXIT: usize = 3;

pub fn syscall(sysnum: usize, mut arg0: usize, arg1: usize, arg2: usize) -> usize {
    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0,
            in("a1") arg1,
            in("a2") arg2,
            in("a3") sysnum
        )
    }

    arg0
}
