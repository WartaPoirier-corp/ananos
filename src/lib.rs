#![cfg_attr(test, no_main)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    alloc_error_handler,
    const_in_array_repeat_expressions,
    const_fn,
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]

extern crate alloc;

use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod serial;
pub mod vga;

pub fn init() {
    gdt::init();
    interrupt::init_idt();
    unsafe { interrupt::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for (i, test) in tests.iter().enumerate() {
        serial_print!("Test {}/{}   ", i + 1, tests.len());
        test();
        serial_println!("[OK]")
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[FAIL]");
    serial_println!("ERROR : {}", info);
    exit_qemu(QemuExitCode::Failure);
    halt_loop()
}

pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
bootloader::entry_point!(kernel_main_test);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn kernel_main_test(_boot_info: &'static bootloader::BootInfo) -> ! {
    init();
    test_main();
    halt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
