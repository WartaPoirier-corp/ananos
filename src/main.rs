#![feature(custom_test_frameworks, abi_x86_interrupt, asm)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

extern crate alloc;

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
    use x86_64::VirtAddr;
    use os::memory;

    println!("Bienvenue dans ananOS !");
    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    os::allocator::init_heap(&mut mapper, &mut frame_allocator).unwrap();

    let x = alloc::boxed::Box::new(42);
    println!("box: {}", x);
    
    os::db::init();
    {
        let mut db = os::db::DB.lock();
        if let Some(datab) = db.as_mut() {
            let handle = datab.open(os::db::Type::byte_type(), datab.find_memory_location());
            let num = datab.read(handle);
            println!("read from DB: {}", num[0]);
        }
    }

    let mut sched = os::process::scheduler::Scheduler::new();
    let proc = os::process::Process::new(
        alloc::vec![0x90, 0xcd, 0x80],
        phys_mem_offset,
        &boot_info.memory_map,
        &mut mapper,
        &mut frame_allocator
    );
    println!("Adding a new process");
    sched.spawn(proc);
    println!("Going to usermode");
    sched.next();


    let mut exec = os::task::executor::Executor::new();
    exec.spawn(os::task::Task::new(example_task()));
    exec.spawn(os::task::Task::new(os::task::keyboard::print_keypresses()));
    exec.run();

    #[cfg(test)]
    test_main();

    // os::halt_loop();
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
