pub mod uart8250;
pub mod uart16650;

use spin::{Once, mutex::SpinMutex};
pub use uart8250::{Uart as Uart8250};
pub use uart16650::{Uart as Uart16650};

#[derive(Debug)]
pub struct UartInitError;

// TODO: Remove once allocated drivers are working.
pub static UART8250: Once<SpinMutex<Uart8250>> = Once::new();
pub static UART16650: Once<SpinMutex<Uart16650>> = Once::new();


