#![no_std]
#![no_main]
use core::panic::PanicInfo;
mod vga_buffer;

#[unsafe(no_mangle)] // dont generate a random name for the function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    panic!("Some panic message");
    loop {}
}

// this function is called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

