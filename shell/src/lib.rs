#![no_std]
#![feature(ascii_char, ascii_char_variants)]

use core::ascii::Char;

use crate::syscall::*;

mod syscall;
pub mod print;

pub struct Shell {
    buffer: [Char; 80],
    size: usize
}

impl Default for Shell {
    fn default() -> Self {
        Self { buffer: [Char::Null; 80], size: 0 }
    }
}

impl Shell {
    pub fn enter(&mut self) {
        loop {
            if self.size >= 80 {
                println!("\rMax Size allotted. Resetting input. Press any character to continue.             ");
                self.reset();
                getchar();
                continue;
            }
            print!("\r> {}", self.buffer.as_str());

            let char: Char = match Char::from_u8(getchar()) {
                Some(Char::CarriageReturn | Char::LineFeed) =>  {
                    let command_result = self.run_command();
                    self.reset();
                    println!();
                    continue;
                },
                Some(char) => char,
                None => continue,
            };

            self.buffer[self.size] = char;
            self.size += 1;
        }
    }

    fn run_command(&mut self) {
        let command: &str = (&self.buffer[0..self.size]).as_str();
        let mut command_split = command.split_whitespace();
        let comm = command_split.next().unwrap();

        println!();
        match comm {
            "hello" => print!("Hello!"),
            "exit" => exit(),
            "read" => {
                let mut buf = [0u8; 76];
                read(command_split.next().unwrap(), &mut buf);
                print!("{}", str::from_utf8(&buf).unwrap());
            }
            _ => print!("Invalid command. Please try again."),
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[unsafe(no_mangle)]
pub fn putchar(char: u8) {
    syscall(SYS_PUTCHAR, char as usize, 0, 0, 0);
}

#[unsafe(no_mangle)]
pub fn getchar() -> u8 {
    match syscall(SYS_GETCHAR, 0, 0, 0, 0) {
        FileResult::Ok(val) => val as u8,
        FileResult::Err(file_err) => panic!(),
    }
}

#[unsafe(no_mangle)]
pub fn exit() -> ! {
    syscall(SYS_EXIT, 0, 0, 0, 0);
    loop {}
}

pub fn read(name: &str, buf: &mut [u8]) -> FileResult {
    let str_ptr = name.as_ptr() as usize;
    let str_len = name.len();

    let buf_ptr = buf.as_ptr() as usize;
    let buf_len = buf.len();

    syscall(SYS_READ, str_ptr, str_len, buf_ptr, buf_len)
}
