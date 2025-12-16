use core::arch::asm;

#[repr(i64)]
pub enum ImplId {
    BerkeleyBootLoader = 0,
    OpenSBI            = 1,
    Xvisor             = 2,
    KVM                = 3,
    RustSBI            = 4,
    Diosix             = 5,
    Coffer             = 6,
}

#[rustfmt::skip]
#[repr(i64)]
pub enum HartState {
    Started        = 0,
    Stopped        = 1,
    StartPending   = 2,
    StopPending    = 3,
    Suspended      = 4,
    SuspendPending = 5,
    ResumePending  = 6,
}

#[repr(C, i64)]
#[rustfmt::skip]
pub enum SbiRet {
    SbiSuccess { value: i64 } =  0,
    SbiErrFailed              = -1,
    SbiErrNotSupported        = -2,
    SbiErrInvalidParam        = -3,
    SbiErrDenied              = -4,
    SbiErrInvalidAddress      = -5,
    SbiErrAlreadyAvailable    = -6,
    SbiErrAlreadyStarted      = -7,
    SbiErrAlreadyStopped      = -8,
}

#[repr(C)]
pub struct SbiRetInto {
    error: i64,
    value: i64,
}

#[rustfmt::skip]
unsafe fn sbi_call(
    mut arg0: u64,
    mut arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    fid: u64,
    eid: u64
) -> SbiRet {
    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0,
            inout("a1") arg1,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") eid,
        )
    }

    unsafe {
        core::mem::transmute(
            SbiRetInto {
                error: arg0.cast_signed(),
                value: arg1.cast_signed(),
            }
        )
    }
}

/// Returns the imlementation ID
#[allow(dead_code)]
pub fn sbi_get_impl_id() -> crate::sbi::SbiRet {
    unsafe {
        sbi_call(0,0,0,0,0,0,0x1,0x10)
    }
}

/// Legacy call (incompatiable with [`sbi_call`])
pub fn sbi_getchar() -> i64 {
    let mut arg0: i64 = 0;
    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0,
            in("a1") 0,
            in("a2") 0,
            in("a3") 0,
            in("a4") 0,
            in("a5") 0,
            in("a6") 0,
            in("a7") 2,
        )
    }

    arg0
}

macro_rules! define_sbi_fn {
    ($name:ident, $fid:literal, $eid:literal, $(,)? $(#[$attr:meta])*) => {
        #[allow(dead_code)]
        $(#[$attr])*
        pub fn $name() -> $crate::sbi::SbiRet {
            unsafe {
                sbi_call(0, 0, 0, 0, 0, 0, $fid, $eid)
            }
        }
    };
    ($name:ident, $fid:literal, $eid:literal, $arg:ident: $ty:ty, $(,)? $(#[$attr:meta])*) => {
        #[allow(dead_code)]
        $(#[$attr])*
        pub fn $name($arg: $ty) -> $crate::sbi::SbiRet {
            unsafe {
                sbi_call($arg as u64, 0, 0, 0, 0, 0, $fid, $eid)
            }
        }
    };
    ($name:ident, $fid:literal, $eid:literal, $arg0:ident: $ty0:ty, $arg1:ident: $ty1:ty, $(,)?  $(#[$attr:meta])*) => {
        #[allow(dead_code)]
        $(#[$attr])*
        pub fn $name($arg0: $ty0, $arg1: $ty1) -> $crate::sbi::SbiRet {
            unsafe {
                sbi_call($arg0 as u64, $arg1 as u64, 0, 0, 0, 0, $fid, $eid)
            }
        }
    };
    ($name:ident, $fid:literal, $eid:literal, $arg0:ident: $ty0:ty, $arg1:ident: $ty1:ty, $arg2:ident: $ty2:ty, $(,)? $(#[$attr:meta])*) => {
        #[allow(dead_code)]
        $(#[$attr])*
        pub fn $name($arg0: $ty0, $arg1: $ty1, $arg2: $ty2) -> $crate::sbi::SbiRet {
            unsafe {
                sbi_call($arg0 as u64, $arg1 as u64, $arg2 as u64, 0, 0, 0, $fid, $eid)
            }
        }
    };
}

macro_rules! sbi_fns {
    ($([$($args:tt)*]),* $(,)?) => {
        $(
            define_sbi_fn!($($args)*);
        )*
    };
}

sbi_fns!(
    [sbi_putchar, 0, 1, ch: u8,
        /// Legacy call (compatiable with [`sbi_call`])
    ],
    [sbi_get_spec_version, 0x0, 0x10,
        /// Returns the current SBI version. Must succeed
    ],
    [sbi_probe_extension, 3, 0x10, extension_id: u64,
        /// Returns 0 if EID extension_id is not available,
        /// and 1 if it is, unless defined by impl as some
        /// other non-zero value
    ],
    [sbi_get_mvendorid, 4, 0x10,
        /// Returns legal value for `mvendorid`. 0 is always valid
    ],
    [sbi_get_marchid, 5, 0x10,
        /// Returns legal value for `marchid`. 0 is always valid
    ],
    [sbi_get_mimpid, 6, 0x10,
        /// Returns legal value for `mimpid`. 0 is always valid
    ],
    [sbi_set_timer, 0, 0x54494D45, stime_value: u64,
        /// Sets the `stime` csr
    ],
    [sbi_send_ipi, 0, 0x735049, hart_mask: u64, hart_mask_base: u64,
        /// Sends an inter-process interrupt to all harts in `hart_mask`.
        /// These are recieved as software interrupts.
    ],
    // Don't wanna extend macro rn but this might be useful at some point
    // [sbi_remote_fence_i, 0, 0x52464E43],
    // [sbi_remote_sfence_vma, 1, 0x52464E43],
    // [sbi_remote_sfence_vma_asid, 2, 0x52464E43],
    // [sbi_remote_hfence_gvma_vmid, 3, 0x52464E43],
    // [sbi_remote_hfence_gvma, 4, 0x52464E43],
    // [sbi_remote_hfence_vvma_asid, 5, 0x52464E43],
    // [sbi_remote_hfence_vvma, 6, 0x52464E43],
    [sbi_hart_start, 0, 0x48534D, hartid: u64, start_addr: usize, opaque: u64,
        /// Begin executing `hartid` at `start_addr` in supervisor mode.
        /// `hartid` will be in register a0, and `opaque` in a1
    ],
    [sbi_hart_stop, 1, 0x48534D,
        /// Stops execution of hart and return ownership to SBI
        /// The sbi_hart_stop() must be called with the supervisor-mode interrupts disabled.
    ],
    [sbi_hart_get_status, 2, 0x48534D, hart_id: u64,
        /// Gets current [`HartState`] or returns [`SbiRet::SbiErrInvalidParam`]
    ],
    [sbi_hart_suspend, 3, 0x48534D, suspend_type: u32, resume_addr: u64, opaque: u64,
        /// Suspends the hart. Returning from a non-retentive suspend, the hart resumes
        /// similar to the [`sbi_hart_start`] SBI call
    ],
    [sbi_system_reset, 0, 0x53525354, reset_type: u32, reset_reason: u32,
        /// Reset the cpu. Does not return on success.
    ]
);
