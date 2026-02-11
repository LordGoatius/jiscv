use core::ptr::NonNull;

use owo_colors::{OwoColorize, colors::*};
use spin::{Mutex, MutexGuard, Once};

use crate::{registers::{ReadWrite, Register}, traits::KSay};

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

struct Uart(NonNull<Registers>);

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
    lsr: Register<ReadWrite, u8>,
    /// msr: +6, Read, Modem Status Register
    msr: Register<ReadWrite, u8>,
    /// sr: +7, R/W, Scratch Register
    sr: Register<ReadWrite, u8>,
}

unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

static UART: Once<Mutex<Uart>> = Once::new();

impl Uart {
    fn from_ptr(ptr: *mut u8) -> Result<Mutex<Uart>, UartInitError> {
        if ptr.is_null() {
            Err(UartInitError)
        } else {
            unsafe { Result::Ok(Mutex::new(Uart(NonNull::new_unchecked(ptr.cast())))) }
        }
    }

    pub fn write_str(&self, str: &str) {
        todo!("{:?}", self.0)
    }
}

pub struct UartInitError;

// for i in "hello".bytes() {
//     unsafe {
//         (0x1000_0000 as *mut u8).write_volatile(i);
//     }
// }

#[inline(never)]
pub fn init_uart(addr: *mut u8) -> Result<(), UartInitError> {
    let uart = UART.try_call_once(|| Uart::from_ptr(addr));

    match uart {
        Ok(_) => <MutexGuard<'_, Uart> as KSay>::kprint("UART successfully initalized".fg::<Green>()),
        Err(_) => <MutexGuard<'_, Uart> as KSay>::kprint("UART failed to initalize".fg::<Red>()),
    }

    uart.map(|_| ())
}

impl KSay for MutexGuard<'_, Uart>  {
    const NAME: &'static str = "serial";
}
