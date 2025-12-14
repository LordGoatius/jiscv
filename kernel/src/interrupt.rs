use crate::{println, read_csr, trap::SCAUSE_INT, user::SSTATUS_SPIE, write_csr};

#[macro_use]
pub mod macros {
    #[macro_export]
    macro_rules! read_cycle {
        () => {{
            let mut value: usize;
            unsafe {
                ::core::arch::asm!("rdcycle {}", out(reg) value);
            }
            value
        }};
    }

    #[macro_export]
    macro_rules! read_time {
        () => {{
            let mut value: usize;
            unsafe {
                ::core::arch::asm!("rdtime {}", out(reg) value);
            }
            value
        }};
    }

    #[macro_export]
    macro_rules! read_instret {
        () => {{
            let mut value: usize;
            unsafe {
                ::core::arch::asm!("rdinstret {}", out(reg) value);
            }
            value
        }};
    }
}

/// Enables external interrupts
pub const SIE_SUPERVISOR_EXTERNAL_INTERRUPT_ENABLE: usize = 1 << 9;
/// Enables timer interrupts
pub const SIE_TIMER_EXTERNAL_INTERRUPT_ENABLE: usize = 1 << 5;
/// Enables software interrupts
pub const SIE_SOFTWARE_EXTERNAL_INTERRUPT_ENABLE: usize = 1 << 1;
/// Enables SIE interrupts as supervisor
pub const SSTATUS_SIE: usize = 1 << 1;

#[unsafe(no_mangle)]
pub fn interrupt_enable() {
    write_csr!(
        "sie",
        SIE_TIMER_EXTERNAL_INTERRUPT_ENABLE
            | SIE_SUPERVISOR_EXTERNAL_INTERRUPT_ENABLE
            | SIE_SOFTWARE_EXTERNAL_INTERRUPT_ENABLE
    );

    // this is about 2 seconds?
    write_csr!("stimecmp", 0xffffff);
}

pub fn handle_interrupt(scause: usize, sepc: usize, stval: usize) {
    let scause_readable = match scause & !SCAUSE_INT {
        0 => "Reserved",
        1 => "Supervisor software interrupt",
        2..=4 => "Reserved",
        5 => "Supervisor timer interrupt",
        6..=8 => "Reserved",
        9 => "Supervisor external interrupt",
        10..=12 => "Reserved",
        13 => "Counter-overflow interrupt",
        14..=15 => "Reserved",
        16.. => "Designated for platform use",
    };

    // println!(
    //     "interrupt handler: {} at {:#x} (stval={:#x})",
    //     scause_readable, sepc, stval
    // );

    match scause {
        0x8000000000000005 => {
            let mtime = read_time!();
            write_csr!("stimecmp", mtime + 0xffffff);
        }
        _ => (),
    }
}
