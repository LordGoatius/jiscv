#![expect(dead_code)]
use core::fmt::{Debug, Write};

use owo_colors::{OwoColorize, colors::*};
use spin::{Once, mutex::SpinMutex};

use crate::{print::{Printer, set_printer}, registers::*, traits::KSay};

// From Wikibooks
// (https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming#UART_Registers):
// |Base Addr | DLAB    | I/O Access | Abbrv.  | Register Name                    |
// |----------|---------|------------|---------|----------------------------------|
// |+0        | 0       | Write      | THR     | Transmitter Holding Buffer       |
// |+0        | 0       | Read       | RBR     | Receiver Buffer                  |
// |+0        | 1       | Read/Write | DLL     | Divisor Latch Low Byte           |
// |+1        | 0       | Read/Write | IER     | Interrupt Enable Register        |
// |+1        | 1       | Read/Write | DLH     | Divisor Latch High Byte          |
// |+2        | x       | Read       | IIR     | Interrupt Identification Register|
// |+2        | x       | Write      | FCR     | FIFO Control Register            |
// |+3        | x       | Read/Write | LCR     | Line Control Register            |
// |+4        | x       | Read/Write | MCR     | Modem Control Register           |
// |+5        | x       | Read       | LSR     | Line Status Register             |
// |+6        | x       | Read       | MSR     | Modem Status Register            |
// |+7        | x       | Read/Write | SR      | Scratch Register                 |
// Shoutout Wikibooks

/// The infrastructre to write a full serial driver sure does exist now.
/// Maybe I'll even do it some day.
/// It's not complicated but now that it does what I want I wanna do the more fun
/// stuff I can do in the os.
pub struct Uart(&'static mut Registers);

impl Printer for Uart {
    fn name(&self) -> &str { "uart" }
}

#[derive(Debug)]
struct Registers {
    /// thr: +0, Write, Transmitter Holding Buffer
    /// rbr: +0, Read, Receiver Buffer
    /// dll: +0, R/W, Divisor Latch Low
    thr_rbr_dll: Register<ReadWrite, u8>,
    /// ier: +1, R/W, Interrupt Enable Register
    /// dlh: +1, R/W, Divisor Latch High
    ier_dlh: Register<ReadWrite, u8>,
    /// iir: +2, Read, Interrupt Identification Register
    /// fcr: +2, Write FIFO Control Register
    iir_fcr: Register<ReadWrite, u8>,
    /// lcr: +3, R/W, Line Control Register
    lcr: Register<ReadWrite, u8>,
    /// mcr: +4, R/W, Modem Control Register
    mcr: Register<ReadWrite, u8>,
    /// lsr: +5, Read, Line Status Register
    lsr: Register<Read, u8>,
    /// msr: +6, Read, Modem Status Register
    msr: Register<Read, u8>,
    /// sr: +7, R/W, Scratch Register
    sr: Register<ReadWrite, u8>,
}

unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

// TODO: Remove once allocated drivers are working.
pub static UART: Once<SpinMutex<Uart>> = Once::new();

impl Uart {
    fn from_ptr(ptr: *mut u8) -> Result<SpinMutex<Uart>, UartInitError> {
        if ptr.is_null() {
            Err(UartInitError)
        } else {
            unsafe {
                Result::Ok(SpinMutex::new(Uart(ptr.cast::<Registers>().as_mut_unchecked())))
            }
        }
    }

    pub fn set_printer(&self) {
        // # Safety:
        // Uh well self has to be the single static UART driver
        // if it's not then wtf we doing here
        // I gotta allocate my drivers man
        unsafe {
            // Was easier than making the function only work with statics
            // Okay so uh
            // Maybe I can do this. Maybe I am special.
            set_printer((self as *const dyn Printer).cast_mut().as_mut_unchecked());
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            // uh just shove those bits through without a care in the world
            // no need for "validating logic"
            self.0.thr_rbr_dll.write(byte);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct UartInitError;

#[inline(never)]
pub fn init_uart(addr: *mut u8) -> Result<&'static SpinMutex<Uart>, UartInitError> {
    let uart = UART.try_call_once(|| Uart::from_ptr(addr));

    match uart {
        Ok(_) => <Uart as KSay>::kprint("UART successfully created".fg::<Green>()),
        Err(_) => <Uart as KSay>::kprint("UART failed to initalize".fg::<Red>()),
    }

    uart
}

impl KSay for Uart  {
    const NAME: &'static str = "serial";
}
