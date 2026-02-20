pub mod uart8250;
pub mod uart16550;

use spin::{Once, mutex::SpinMutex};
pub use uart8250::{Uart as Uart8250};
pub use uart16550::{Uart as Uart16550};

#[derive(Debug)]
pub struct UartInitError;

// TODO: Remove once allocated drivers are working.
pub static UART8250: Once<SpinMutex<Uart8250>> = Once::new();
pub static UART16550: Once<SpinMutex<Uart16550>> = Once::new();


