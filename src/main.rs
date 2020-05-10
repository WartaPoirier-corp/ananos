#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Bienvenue dans ananOS !");
    loop {}
}
