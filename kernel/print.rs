use core::fmt::Write;

use owo_colors::{colors::*, OwoColorize};

// Okay so I want SBI putchar as a (blocking) alternative to UART communication
// But I want every macro to work.
// Solution:
// - Dynamic dispatch of a static in the print module that could be either
// a [`crate::print::SbiPrinter`] or a [`crate::uart::Uart`], which both implement
// [`crate::print::Printer`], which depends on the internal [`core::fmt::Write`]
// implementation for writing.
pub struct SbiPrinter;

pub trait Printer: Write {
    fn name(&self) -> &str;
}

impl Printer for SbiPrinter {
    fn name(&self) -> &str {
        "sbi-printer"
    }
}

static mut SBI_PRINTER: SbiPrinter = SbiPrinter;

/// NOTE: DO NOT USE
pub static mut PRINTER: &mut dyn Printer = unsafe { &mut SBI_PRINTER as &mut dyn Printer };

pub fn set_printer(printer: &'static mut dyn Printer) {
    crate::println!(
        "[{}]: printer changed to: {}",
        "printer".fg::<Yellow>(),
        printer.name().fg::<BrightCyan>()
    );
    unsafe {
        PRINTER = printer;
    }
}

impl core::fmt::Write for SbiPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            crate::sbi::sbi_putchar(b);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            #[allow(unused)]
            use ::core::fmt::Write;
            let _ = writeln!($crate::print::PRINTER, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            #[allow(unused)]
            use ::core::fmt::Write;
            let _ = write!($crate::print::PRINTER, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! dbg {
    ($($arg:tt)*) => {{
        let val = $($arg)*;
        println!("{} : {:?}", stringify!($($arg)*), val);
        val
    }};
}
