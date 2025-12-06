use core::arch::{asm, naked_asm};

use crate::{PROC_CURR, PROC_IDLE, println, write_csr};

const PROC_MAX: usize = 0x16;
const STACK_SIZE: usize = 0x2000;
pub static mut PROCS: [Process; PROC_MAX] = [Process::default(); PROC_MAX];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Process {
    pub(crate) pid: usize,
    pub(crate) state: ProcessState,
    pub(crate) sp: usize,
    pub(crate) kstack: [u8; STACK_SIZE],
}

impl const Default for Process {
    fn default() -> Self {
        Self {
            pid: usize::MAX,
            state: ProcessState::Unused,
            sp: usize::MAX,
            kstack: [0; STACK_SIZE],
        }
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Unused,
    InUse,
}

impl const Default for ProcessState {
    fn default() -> Self {
        Self::Unused
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum ProcessError {
    MaxProcsUsed = 0,
}

pub fn r#yield() {
    let mut next = unsafe {
        *PROC_IDLE
    };

    for i in 0..PROC_MAX {
        let proc = unsafe {
            &raw mut PROCS[
                ((*PROC_CURR.unwrap()).pid + i + 1) % PROC_MAX
            ]
        };
        if unsafe { (*proc).state == ProcessState::InUse && (*proc).pid > 0 } {
            next = proc;
            break;
        }
    }

    if unsafe { next == PROC_CURR.unwrap() } {
        // Do nothing
        return;
    }


    unsafe {
        write_csr!("sscratch", (*next).kstack.as_mut_ptr().add(STACK_SIZE) as u64);

        let prev = PROC_CURR.unwrap();
        PROC_CURR = Some(next);
        switch_context(&raw mut (*prev).sp, &raw mut (*next).sp);
    }
}

pub fn create_process(pc: usize) -> Result<*mut Process, ProcessError> {
    let mut index = None;
    for i in 0..PROC_MAX {
        if unsafe { PROCS[i] }.state == ProcessState::Unused {
            index = Some(i);
            break;
        }
    }

    index
        .map(|proc| unsafe {
            let ptr = &raw mut PROCS[proc];
            let mut sp = &raw mut PROCS[proc].kstack[STACK_SIZE - 8] as *mut usize;

            for _ in 0..12 {
                sp.write(0);
                sp = sp.wrapping_sub(1);
            }

            sp.write(pc);

            (*ptr).pid = proc;
            (*ptr).state = ProcessState::InUse;
            (*ptr).sp = sp as usize;

            ptr
        })
        .ok_or_else(|| ProcessError::MaxProcsUsed)
}

#[unsafe(naked)]
pub extern "C" fn switch_context(sp_prev: *mut usize, sp_new: *mut usize) {
    naked_asm!(
        // Save callee-saved registers onto the current process's stack.
        "addi sp, sp, (-13 * 8)", // Allocate stack space for 13 8-byte registers
        "sd ra,  0  * 8(sp)",   // Save callee-saved registers only
        "sd s0,  1  * 8(sp)",
        "sd s1,  2  * 8(sp)",
        "sd s2,  3  * 8(sp)",
        "sd s3,  4  * 8(sp)",
        "sd s4,  5  * 8(sp)",
        "sd s5,  6  * 8(sp)",
        "sd s6,  7  * 8(sp)",
        "sd s7,  8  * 8(sp)",
        "sd s8,  9  * 8(sp)",
        "sd s9,  10 * 8(sp)",
        "sd s10, 11 * 8(sp)",
        "sd s11, 12 * 8(sp)",
        // Switch the stack pointer.
        "sd sp, (a0)", // *prev_sp = sp;
        "ld sp, (a1)", // Switch stack pointer (sp) here (sp = *sp_new)
        // Restore callee-saved registers from the next process's stack.
        "ld ra,  0  * 8(sp)", // Restore callee-saved registers only
        "ld s0,  1  * 8(sp)",
        "ld s1,  2  * 8(sp)",
        "ld s2,  3  * 8(sp)",
        "ld s3,  4  * 8(sp)",
        "ld s4,  5  * 8(sp)",
        "ld s5,  6  * 8(sp)",
        "ld s6,  7  * 8(sp)",
        "ld s7,  8  * 8(sp)",
        "ld s8,  9  * 8(sp)",
        "ld s9,  10 * 8(sp)",
        "ld s10, 11 * 8(sp)",
        "ld s11, 12 * 8(sp)",
        "addi sp, sp, 13 * 8", // We've popped 13 4-byte registers from the stack
        "ret",
    )
}
