use core::arch::asm;

pub const SYS_PUTCHAR: usize = 1;
pub const SYS_GETCHAR: usize = 2;
pub const SYS_EXIT: usize = 3;
pub const SYS_WRITE: usize = 4;
pub const SYS_READ: usize = 5;

#[derive(Debug)]
#[repr(isize)]
pub enum FileResult {
    Ok(usize) = 0,
    Err(FileErr) = -1,
}

#[derive(Debug)]
#[repr(usize)]
pub enum FileErr {
    FileNotFound, 
    BufferTooLarge
}

pub fn syscall(sysnum: usize, mut arg0: usize, mut arg1: usize, arg2: usize, arg3: usize) -> FileResult {
    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0,
            inout("a1") arg1,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") sysnum
        )
    }

    unsafe { core::mem::transmute([arg0, arg1]) }
}
