use core::fmt::{Debug, Write};

use owo_colors::{OwoColorize, colors::*};
use spin::mutex::SpinMutex;

use crate::{
    print::{Printer, set_printer},
    registers::*,
    traits::KSay,
    uart::{UART16650, UartInitError}
};

// | Address | Register | Access Type | Reset Value | Description
// |---------|----------|-------------|-------------|-------------
// | 0x00    | RBR      | Read only   | 0x00        | Receiver Buffer Register
// | 0x00    | THR      | Write only  | 0x00        | Transmitter Holding Register
// | 0x01    | IER      | Read/Write  | 0x00        | Enable(1)/Disable(0) interrupts. See this for more details on each interrupt.
// | 0x02    | IIR      | Read only   | 0x01        | Information which interrupt occurred
// | 0x02    | FCR      | Write only  | 0x00        | Control behavior of the internal FIFOs. Currently writing to this Register has no effect.
// | 0x03    | LCR      | Read/Write  | 0x00        | The only bit in this register that has any meaning is LCR7 aka the DLAB, all other bits hold their written value but have no meaning.
// | 0x05    | LSR      | Read only   | 0x60        | Information about state of the UART. After the UART is reset, 0x60 indicates when it is ready to transmit data.

pub struct Uart(&'static mut Registers);

impl Printer for Uart {
    fn name(&self) -> &str { "uart16650" }
}

#[repr(C)]
#[derive(Debug)]
struct Registers {
    rbr_thr: Register<ReadWrite, u8>,
    ier: Register<ReadWrite, u8>,
    iir_fcr: Register<ReadWrite, u8>,
    lcr: Register<ReadWrite, u8>,
    _unused: u8,
    lsr: Register<Read, u8>
}

unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

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
            self.0.rbr_thr.write(byte);
        }
        Ok(())
    }
}

#[inline(never)]
pub fn init_uart_16650(addr: *mut u8) -> Result<&'static SpinMutex<Uart>, UartInitError> {
    let uart = UART16650.try_call_once(|| Uart::from_ptr(addr));

    match uart {
        Ok(_) => <Uart as KSay>::kprint("UART successfully created".fg::<Green>()),
        Err(_) => <Uart as KSay>::kprint("UART failed to initalize".fg::<Red>()),
    }

    uart
}

impl KSay for Uart  {
    const NAME: &'static str = "uart16650";
}
