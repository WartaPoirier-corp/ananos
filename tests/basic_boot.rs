#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::{println, serial_print, serial_println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[test_case]
fn test_println() {
    serial_print!("test_println... ");
    println!("test_println output");
    serial_println!("[ok]");
}
