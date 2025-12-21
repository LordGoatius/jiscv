use crate::{PROC_CURR, proc::{Process, ProcessState, r#yield}, sbi::{sbi_getchar, sbi_putchar}, trap::TrapFrame};

pub const SYS_PUTCHAR: usize = 1;
pub const SYS_GETCHAR: usize = 2;
pub const SYS_EXIT: usize = 3;
pub const SYS_WRITE: usize = 4;
pub const SYS_READ: usize = 5;

pub fn handle_syscall(f: &mut TrapFrame) {
    match f.a3 {
        SYS_PUTCHAR => {
            sbi_putchar(f.a0 as u8);
            ()
        }
        SYS_GETCHAR => loop {
            let char = sbi_getchar();
            if char >= 0 {
                f.a0 = char as usize;
                break;
            }
            r#yield();
        },
        SYS_EXIT => {
            let curr_proc: &mut Process = unsafe { PROC_CURR.unwrap().as_mut().unwrap() };
            println!("Process exiting: {}", curr_proc.pid);
            curr_proc.state = ProcessState::Exited;
            r#yield();
        }
        call => panic!("Unimplemented syscall {}", call),
    }
}

