use core::{slice, str};

use crate::{PROC_CURR, get_fs_unwrap, proc::{Process, ProcessState, r#yield}, sbi::{sbi_getchar, sbi_putchar}, tar, trap::TrapFrame};

pub const SYS_PUTCHAR: usize = 1;
pub const SYS_GETCHAR: usize = 2;
pub const SYS_EXIT: usize = 3;
pub const SYS_WRITE: usize = 4;
pub const SYS_READ: usize = 5;

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
            let name_ptr = f.a0 as *const u8;
            let name_size = f.a1 as usize;
            // # Safety:
            // The shell syscall wraps taking in a rust &str and deconstructs it
            // behind the scenes, exposing only a safe interface where safe
            // rust can verify a valid &str
            let name = unsafe { str::from_raw_parts(name_ptr, name_size) };

            let buf_ptr = f.a2 as *const u8;
            let buf_size = f.a3 as usize;
            // # Safety:
            // The shell syscall wraps taking in a rust &[u8] and deconstructs it
            // behind the scenes, exposing only a safe interface where safe
            // rust can verify a valid &[u8]
            let buf = unsafe { slice::from_raw_parts(buf_ptr, buf_size) };
        }
        SYS_READ => {
            let name_ptr = f.a0 as *const u8;
            let name_size = f.a1 as usize;
            // # Safety:
            // The shell syscall wraps taking in a rust &str and deconstructs it
            // behind the scenes, exposing only a safe interface where safe
            // rust can verify a valid &str
            let name = unsafe { str::from_raw_parts(name_ptr, name_size) };

            let buf_ptr = f.a2 as *mut u8;
            let buf_size = f.a3 as usize;
            // # Safety:
            // The shell syscall wraps taking in a rust &[u8] and deconstructs it
            // behind the scenes, exposing only a safe interface where safe
            // rust can verify a valid &[u8]
            let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, buf_size) };

            let res = tar::read(get_fs_unwrap(), name, buf);

            let arr: [usize; 2] = unsafe { core::mem::transmute(res) };
            f.a0 = arr[0];
            f.a1 = arr[1];
        }
        call => panic!("Unimplemented syscall {}", call),
    }
}

