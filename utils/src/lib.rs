#![no_std]
#![feature(
    const_array,
    const_trait_impl,
    maybe_uninit_array_assume_init,
    try_trait_v2,
    try_trait_v2_residual
)]

pub mod static_alloc;

pub mod syscall {
    use core::ops::{ControlFlow, FromResidual, Try};

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
        OutOfMemory = -3
    }

    impl Try for SyscallResult {
        type Output = SysOk;

        // May need to refactor for `SyscallResult<Infalliable, SysErr>` when made generic
        type Residual = SysErr;

        fn from_output(output: Self::Output) -> SyscallResult {
            SyscallResult::Ok(output)
        }

        fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
            match self {
                SyscallResult::Ok(val) => ControlFlow::Continue(val),
                SyscallResult::Err(err) => ControlFlow::Break(err),
            }
        }
    }

    impl FromResidual<SysErr> for SyscallResult {
        fn from_residual(residual: SysErr) -> SyscallResult {
            SyscallResult::Err(residual)
        }
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
