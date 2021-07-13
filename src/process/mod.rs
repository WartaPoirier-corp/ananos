use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{
    Page, PageTableFlags,
};
use x86_64::structures::{
    paging::{Mapper, FrameAllocator, Size4KiB},
};
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use crate::gdt::GDT;

// TODO: use virtual memory better, i.e don't map all
// processes in the same page table directory
static STACK_ADDR: AtomicU64 = AtomicU64::new(0x600_000);
static CODE_ADDR: AtomicU64 = AtomicU64::new(0x400_000);

pub struct Process {
    stack_addr: u64,
    code_addr: u64,
}

impl Process {
    pub fn create(
        mapper: &mut impl Mapper<Size4KiB>,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
        asm: &[u8]
    ) -> Process {
        const PAGE_SIZE: u64 = 1024 * 4; 
        let frame = frame_alloc.allocate_frame().unwrap();
        let stack = STACK_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let page = Page::containing_address(VirtAddr::new(stack));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_alloc).unwrap().flush();
        }

        let frame = frame_alloc.allocate_frame().unwrap();
        let code = CODE_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let page = Page::containing_address(VirtAddr::new(code));
        unsafe {
            mapper.map_to(page, frame, flags, frame_alloc).unwrap().flush();
        }

        unsafe {
            let code = code as *mut u8;
            for (i, op) in asm.iter().enumerate() {
                core::ptr::write(code.add(i), *op);
            }
        }

        Process {
            stack_addr: stack,
            code_addr: code,
        }
    }

    pub fn switch(&self) {
        crate::println!("swicthing to userspace");
        let data_sel = GDT.1.user_data_selector.0;
        let code_sel = GDT.1.user_code_selector.0;

        unsafe {
            interrupts::disable();

            asm!(
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",

                "push rax",
                "push rsi",
                
                "pushf",
                "pop rax",
                "or rax, 0x200",
                "push rax",
                
                "push rcx",
                "push rdx",
                "iretq",
                in("rax") data_sel,
                in("rsi") self.stack_addr,
                in("rcx") code_sel,
                in("rdx") self.code_addr,
            );
        }
    }
}
