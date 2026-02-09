use core::ptr::NonNull;

use owo_colors::{OwoColorize, colors::*};
use spin::{Mutex, MutexGuard, Once};

use crate::ksay::KSay;

struct Uart(NonNull<u8>);

unsafe impl Send for Uart {}
unsafe impl Sync for Uart {}

static UART: Once<Mutex<Uart>> = Once::new();

impl Uart {
    fn from_ptr(ptr: *mut u8) -> Result<Mutex<Uart>, UartInitError> {
        if ptr.is_null() {
            Err(UartInitError)
        } else {
            unsafe { Result::Ok(Mutex::new(Uart(NonNull::new_unchecked(ptr)))) }
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
