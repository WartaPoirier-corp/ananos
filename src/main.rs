#![feature(custom_test_frameworks, abi_x86_interrupt, asm)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

use core::{ops::DerefMut, panic::PanicInfo};
use alloc::sync::Arc;
use alloc::string::ToString;
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
    // it also opens a stream of PCI devices and calls the debugger
    // to print info about its state
    let proc = os::process::Process::create(
        &mut mapper,
        &mut frame_allocator,
        include_bytes!("../test.bin"),
    );
    os::process::spawn(proc).unwrap();

    if let bootloader::boot_info::Optional::Some(rsdp) = boot_info.rsdp_addr {
        let acpi_tables = unsafe {
            acpi::AcpiTables::from_rsdp(os::memory::AcpiHandler, rsdp as usize)
        }.unwrap();

        let pci_config_regions = acpi::mcfg::PciConfigRegions::new(&acpi_tables).unwrap();
        let config_access = os::pci::ConfigAccess(pci_config_regions);

        let pci_type = Arc::new(adb::TypeInfo {
            name: "Os.Pci.Device".to_string(),
            id: adb::TypeId(0xC1),
            definition: adb::TypeDef::Product {
                fields: alloc::vec![
                    ("vendor".to_string(), adb::type_ids::TYPE_ID),
                    ("device".to_string(), adb::type_ids::TYPE_ID),
                    ("class".to_string(), adb::type_ids::TYPE_ID),
                    ("subclass".to_string(), adb::type_ids::TYPE_ID),
                ]
            }
        });

        let mut db = os::db::DB.lock();
        for (_address, device) in os::pci::PciResolver::get_info(config_access).devices {
            println!("PCI device: {:04x?}:{:04x?}, 0x{:02x?}/0x{:02x?} ({})", device.vendor_id, device.device_id, device.class, device.sub_class, device.class_info());
            if let Some(datab) = db.as_mut() {
                datab.write_object(adb::DbObject {
                    type_info: alloc::sync::Arc::clone(&pci_type),
                    value: Arc::new(adb::DbValue::Product {
                        fields: alloc::vec![
                            Arc::new(adb::DbValue::U64(device.vendor_id as u64)),
                            Arc::new(adb::DbValue::U64(device.device_id as u64)),
                            Arc::new(adb::DbValue::U64(device.class as u64)),
                            Arc::new(adb::DbValue::U64(device.sub_class as u64)),
                        ]
                    })
                }).unwrap();
            }
        }
        
        if let Some(db) = db.as_mut() {
            os::db::display_contents(db);
        }
    }

    os::ready();

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
