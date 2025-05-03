use alloc::{string::String, vec::Vec};
use crate::print;
use spin::Mutex;
use lazy_static::lazy_static;

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
            _ => {
                print!("error failed to find command");
            }
        }
        stdin.clear();
        print!("\n> ")
    } else {
        stdin.push(c);
        print!("{c}")
    }
}
