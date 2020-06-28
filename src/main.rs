#![feature(custom_test_frameworks, abi_x86_interrupt)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use os::println;

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::halt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    use x86_64::{structures::paging::Page, VirtAddr};
    use os::memory;

    println!("Bienvenue dans ananOS !");
    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(boot_info.memory_map)
    };

    #[cfg(test)]
    test_main();

    os::halt_loop();
}

#[cfg(test)]
#[test_case]
fn trivial_test() {
    assert!(2 + 2 == 4);
}

