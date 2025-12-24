#![no_std]
#![feature(try_trait_v2, try_trait_v2_residual)]

pub mod syscall {
    pub mod consts {
        pub const SYS_PUTCHAR: usize = 1;
        pub const SYS_GETCHAR: usize = 2;
        pub const SYS_EXIT: usize = 3;
        pub const SYS_WRITE: usize = 4;
        pub const SYS_READ: usize = 5;
    }

    // TODO: Make generic for any type `size_of::<T>() == size_of::<[ok variant value]>`
    #[rustfmt::skip]
    #[cfg_attr(target_pointer_width = "64", repr(i32))]
    #[cfg_attr(target_pointer_width = "32", repr(i16))]
    pub enum SyscallResult {
        Ok(SysOk) = 0,
        Err(SysErr) = -1,
    }

    #[cfg(target_pointer_width = "64")]
    pub type SysOk = u32;
    #[cfg(target_pointer_width = "32")]
    pub type SysOk = u16;

    #[cfg_attr(target_pointer_width = "64", repr(i32))]
    #[cfg_attr(target_pointer_width = "32", repr(i16))]
    pub enum SysErr {
        NotFound = -1,
        BufferSize = -2,
    }
}

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

