#![feature(custom_test_frameworks, abi_x86_interrupt, asm)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use core::{ops::DerefMut, panic::PanicInfo};
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

fn kernel_main(boot_info: &'static mut bootloader::BootInfo) -> ! {
    use x86_64::VirtAddr;
    use os::memory;

    if let bootloader::boot_info::Optional::Some(ref fb) = boot_info.framebuffer {
        let fb = fb.buffer();
        let fb_len = fb.len();
        let fb_start = (&fb[0] as *const _) as u64;
        {
            let mut fb = os::FB.lock();
            *fb = Some((fb_start, fb_len));
        }
    }

    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(boot_info.memory_regions.deref_mut())
    };
    os::allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();

    let x = alloc::boxed::Box::new(19);
    println!("box: {}", x);
    
    os::db::init();
    {
        let mut db = os::db::DB.lock();
        if let Some(datab) = db.as_mut() {
            os::db::display_contents(datab);
        }
    }

    let dt = os::cmos::get_datetime();
    println!("Date: {}/{}/{}", dt.day, dt.month, dt.year);
    println!("Time: {}:{}:{}", dt.hours, dt.minutes, dt.seconds);

    {
        use os::task::executor::EXECUTOR;

        let mut exec = EXECUTOR.lock();
        exec.spawn(os::task::Task::new(example_task()));
        exec.spawn(os::task::Task::new(os::task::keyboard::print_keypresses()));
    }

    // simple program that changes the color of the screen
    // with a system call
    let proc = os::process::Process::create(
        &mut mapper,
        &mut frame_allocator,
        &[
            // mov bx, 0x0
            0x66, 0xbb, 0x00, 0x00,
            // mov ax, 0x00
            0x66, 0xb8, 0x00, 0x00,
            // int 0x80 (system call)
            0xcd, 0x80,
            // add bx, 5
            0x66, 0x83, 0xc3, 0x05,
            // jmp to the start of the loop (second line)
            0xeb, 0xf4,
        ]
    );

    if let bootloader::boot_info::Optional::Some(rsdp) = boot_info.rsdp_addr {
        let acpi_tables = unsafe {
            acpi::AcpiTables::from_rsdp(os::memory::AcpiHandler, rsdp as usize)
        }.unwrap();

        let pci_config_regions = acpi::mcfg::PciConfigRegions::new(&acpi_tables).unwrap();
        let config_access = os::pci::ConfigAccess(pci_config_regions);
        os::pci::PciResolver::get_info(config_access);
    }

    proc.switch();

    #[cfg(test)]
    test_main();

    os::halt_loop();
}

#[cfg(test)]
#[test_case]
fn trivial_test() {
    assert!(2 + 2 == 4);
}

// Tests for async

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
