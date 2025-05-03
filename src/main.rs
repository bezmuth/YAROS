#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

/// NOTES
///
/// Im a bit shakey on the entire pages and memory allocation bit, everything
/// else makes sense to me but following the chain of page thingys I do not understand
///
/// I think a page is a space in virtual memory and a frame is the corresponding
/// part of "real" memory
///
/// it also seems like one of my tests is broken but that might be because of
/// the mappers and stuff
///
/// Okay so i think whenever we access memory it will be in the pages (or virt
/// mem) not in the frames, which makes sense I just didnt think about it
/// properly
///
/// https://github.com/vinc/moros  for an example of what I kinda wanna end up with
/// https://github.com/vinc/moros/commits/trunk/?after=94e6038fc5643cbb5159f0ee92c76660cf98b9ab+699 for first few commits
use blog_os::{
    allocator,
    memory::BootInfoFrameAllocator,
    print,
    println,
    task::{keyboard, executor::Executor, Task},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::memory;
    use x86_64::VirtAddr;
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failure");


    print!("> ");
    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::process_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    blog_os::hlt_loop()
}

// this function is called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
