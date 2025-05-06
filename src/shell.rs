use crate::drivers::vga::{clear_buffer, get_cursor_pos, set_cursor_pos};
use crate::drivers::{self, cmos};
use crate::{print, println};
use alloc::{string::String, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

// implement this using async, not lazy_static (maybe?)

lazy_static! {
    pub static ref STDIN: Mutex<String> = Mutex::new(String::new());
}

pub fn handle_key(c: char) {
    let mut stdin = STDIN.lock();
    if c == '\n' {
        print!("\n");
        let split: Vec<&str> = stdin.as_str().split(" ").collect();
        let command = split[0];
        let args = &split[1..];
        match command {
            "help" => {
                print!("i dont know, thats scary");
            }
            "uname" => {
                print!("YANOS: Yet Another Rust Operating System")
            }
            "echo" => {
                for arg in args {
                    print!("{arg} ")
                }
            }
            "time" => {
                let time = cmos::get_time();
                print!("{:02}:{:02}:{:02}", time.hours, time.minutes, time.seconds);
            }
            "clear" => {
                clear_buffer();
            }
            _ => {
                print!("error failed to find command");
            }
        }
        stdin.clear();
        print!("\n> ")
    } else if c as u8 == 0x8 {
        // backspace
        if stdin.len() > 0 {
            let (column, row) = get_cursor_pos();
            set_cursor_pos(column - 1, row);
            print!(" ");
            stdin.pop();
            set_cursor_pos(column - 1, row);
        }
    } else {
        stdin.push(c);
        print!("{c}")
    }
}
