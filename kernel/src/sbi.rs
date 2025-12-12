use core::arch::asm;

#[repr(C)]
pub struct SbiRet {
    error: u64,
    value: u64,
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

    SbiRet {
        error: arg0,
        value: arg1
    }
}

pub fn sbi_putchar(ch: u8) {
    unsafe {
        sbi_call(ch as u64, 0, 0, 0, 0, 0, 0, 1);
    }
}

pub fn sbi_getchar() -> i64 {
    let SbiRet { error, value: _ } = unsafe {
        sbi_call(0, 0, 0, 0, 0, 0, 0, 2)
    };
    return error as i64;
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            sbi_putchar(b);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = writeln!($crate::sbi::Printer, $($arg)*);
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = write!($crate::sbi::Printer, $($arg)*);
    }};
}

