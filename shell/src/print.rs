use crate::putchar;

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            putchar(b);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    () => {{
        use ::core::fmt::Write;
        let _ = write!($crate::print::Printer, "\r\n");
    }};
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = write!($crate::print::Printer, $($arg)*);
        let _ = write!($crate::print::Printer, "\r\n");
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = write!($crate::print::Printer, $($arg)*);
    }};
}

