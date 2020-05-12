#![feature(custom_test_frameworks)]
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
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Bienvenue dans ananOS !");

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(test)]
#[test_case]
fn trivial_test() {
    assert!(2 + 2 == 4);
}

