pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            crate::sbi::sbi_putchar(b);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = writeln!($crate::print::Printer, $($arg)*);
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use ::core::fmt::Write;
        let _ = write!($crate::print::Printer, $($arg)*);
    }};
}

#[macro_export]
macro_rules! dbg {
    ($($arg:tt)*) => {{
        let val = $($arg)*;
        println!("{} : {:?}", stringify!($($arg)*), val);
        val
    }};
}
