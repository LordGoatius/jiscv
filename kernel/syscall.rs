use core::{slice, str};

use crate::{PROC_CURR, proc::{Process, ProcessState, r#yield}, sbi::{sbi_getchar, sbi_putchar}, trap::TrapFrame};

use utils::syscall::consts::*;

pub fn handle_syscall(f: &mut TrapFrame) {
    match f.a4 {
        SYS_PUTCHAR => {
            sbi_putchar(f.a0 as u8);
            f.a0 = 0;
        }
        SYS_GETCHAR => loop {
            let char = sbi_getchar();
            if char >= 0 {
                f.a0 = 0;
                f.a1 = char as usize;
                break;
            }
            r#yield();
        }
        SYS_EXIT => {
            let curr_proc: &mut Process = unsafe { PROC_CURR.unwrap().as_mut().unwrap() };
            println!("Process exiting: {}", curr_proc.pid);
            curr_proc.state = ProcessState::Exited;
            r#yield();
        }
        SYS_WRITE => {
            todo!()
        }
        SYS_READ => {
            todo!()
        }
        call => panic!("Unimplemented syscall {}", call),
    }
}

