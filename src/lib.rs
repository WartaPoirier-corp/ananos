#![cfg_attr(test, no_main)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    alloc_error_handler,
    const_mut_refs
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]

extern crate alloc;

use core::panic::PanicInfo;

pub mod allocator;
pub mod cmos;
pub mod db;
pub mod gdt;
pub mod interrupt;
pub mod memory;
pub mod pci;
pub mod process;
pub mod serial;
pub mod task;

lazy_static::lazy_static! {
    pub static ref FB: spin::Mutex<Option<(u64, usize)>> = spin::Mutex::new(None);
}

pub fn init() {
    gdt::init();
    interrupt::init_idt();
    unsafe {
        let mut pics = interrupt::PICS.lock();
        pics.initialize();
        let [mask1, mask2] = pics.read_masks();
        pics.write_masks(mask1 | 1, mask2);
    };
    x86_64::instructions::interrupts::enable();
}

// Re-enable timer interrupts when the OS is ready
pub fn ready() {
    // interrupts have to be disabled, because as soon
    // as the timer interrupts will be re-enabled, the ISR
    // will be called, and it needs to lock the PICS too
    // to send the "end of interrupt" notification
    // so to avoid a deadlock, we re-enable interrupts only
    // once the lock in this function is dropped
    x86_64::instructions::interrupts::disable();
    unsafe {
        let mut pics = interrupt::PICS.lock();
        let [mask1, mask2] = pics.read_masks();
        pics.write_masks(mask1 ^ 1, mask2);
    }
    x86_64::instructions::interrupts::enable();
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for (i, test) in tests.iter().enumerate() {
        print!("Test {}/{}   ", i + 1, tests.len());
        test();
        println!("[OK]")
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    println!("[FAIL]");
    println!("ERROR : {}", info);
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
fn kernel_main_test(_boot_info: &'static mut bootloader::BootInfo) -> ! {
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
