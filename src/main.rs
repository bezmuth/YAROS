#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yaros::test_runner)]
#![reexport_test_harness_main = "test_main"]

// NOTES
//
// https://github.com/vinc/moros  for an example of what I kinda wanna end up with
// https://github.com/vinc/moros/commits/trunk/?after=94e6038fc5643cbb5159f0ee92c76660cf98b9ab+699 for first few commits
//
// TODO: get a proper vga driver implemented (i.e dynamic colours and cursor
// changing) this will allow for backspace
use yaros::{
    allocator, memory::BootInfoFrameAllocator, print, println, task::{executor::Executor, keyboard, Task}
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use yaros::memory;
    use x86_64::VirtAddr;
    yaros::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failure");

    #[cfg(test)]
    test_main();

    print!("> ");
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::process_keypresses()));
    executor.run();
}

// this function is called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use yaros::println;
    println!("{}", info);
    yaros::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    yaros::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
