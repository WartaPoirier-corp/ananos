use alloc::{boxed::Box, vec::Vec};
use x86_64::{
    VirtAddr, 
    structures::paging::{
        mapper::OffsetPageTable,
        FrameAllocator, Page, PageTable, PageTableFlags,
        Mapper, Size4KiB, PhysFrame,
    },
};

pub mod scheduler;

enum ProcessStatus {
    Ready,
    Started,
    Done,
}

struct Context {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
}

impl Context {
    fn new() -> Context {
        Context {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
        }
    }

    fn get() -> Context {
        let mut ctx = Context::new();
        unsafe {
                asm!(
                "nop",
                out("rax") ctx.rax,
                out("rbx") ctx.rbx,
                out("rcx") ctx.rcx,
                out("rdx") ctx.rdx,
            );
        }
        ctx
    }
}

const KSTACK_SIZE: u64 = 8192;
const STACK_SIZE: u64 = 8192;

pub struct Process<'a> {
    status: ProcessStatus,
    kstack: &'a (),
    stack: &'a (),
    code: &'a (),
    page_table_frame: PhysFrame,
    context: Context,
}

impl<'a> Process<'a> {
    pub fn new(
        code: Vec<u8>,
        offset: VirtAddr,
        mem_regions: &bootloader::bootinfo::MemoryMap,
        kernel_mapper: &mut impl Mapper<Size4KiB>,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
    ) -> Process<'a> {
        use bootloader::bootinfo::MemoryRegionType;
        use x86_64::PhysAddr;

        let mut page_table = PageTable::new();
        let mut mapper = unsafe { OffsetPageTable::new(&mut page_table, offset) };
        for reg in mem_regions.iter().filter(|r| r.region_type == MemoryRegionType::Kernel || r.region_type == MemoryRegionType::KernelStack).map(|r| r.range) {
            for frame in PhysFrame::<Size4KiB>::range(
                PhysFrame::containing_address(PhysAddr::new(reg.start_addr())),
                PhysFrame::containing_address(PhysAddr::new(reg.end_addr())),
            ) {
                let virt_addr = offset + frame.start_address().as_u64();
                // crate::println!("{:?} {:?} {:?}", offset, frame, virt_addr);
                let page = Page::containing_address(virt_addr);
                unsafe { 
                    mapper.map_to(page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_alloc)
                        .unwrap()
                        .ignore();
                }
            }
        }

        const STACK_ADDR : u64 = 0x1000_0000;
        const CODE_ADDR  : u64 = 0x2000_0000;
        const KSTACK_ADDR: u64 = 0x3000_0000;

        let stack_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        let code_flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        let kstack_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let stack_pages = Page::range_inclusive(
            Page::containing_address(VirtAddr::new(STACK_ADDR)),
            Page::containing_address(VirtAddr::new(STACK_ADDR + STACK_SIZE - 1)),
        );
        for page in stack_pages {
            let frame = frame_alloc.allocate_frame().unwrap();
            unsafe {
                mapper.map_to(page, frame, stack_flags, frame_alloc).unwrap().ignore();
            }
        }
        
        let code_pages = Page::range_inclusive(
            Page::containing_address(VirtAddr::new(CODE_ADDR)),
            Page::containing_address(VirtAddr::new(CODE_ADDR + code.len() as u64)),
        );
        for page in code_pages {
            let frame = frame_alloc.allocate_frame().unwrap();
            unsafe {
                kernel_mapper.map_to(page, frame, code_flags | PageTableFlags::WRITABLE, frame_alloc).unwrap().flush();
                mapper.map_to(page, frame, code_flags, frame_alloc).unwrap().ignore();
            }
        }
        
        let code_ptr = CODE_ADDR as *mut u8;
        for (i, byte) in code.iter().enumerate() {
            crate::println!("{} = {:#x}", i, byte);
            unsafe { core::ptr::write(code_ptr.add(i), *byte); }
        }
        for page in code_pages {
            kernel_mapper.unmap(page).unwrap().1.flush();
        }

        let kstack_pages = Page::range_inclusive(
            Page::containing_address(VirtAddr::new(KSTACK_ADDR)),
            Page::containing_address(VirtAddr::new(KSTACK_ADDR + KSTACK_SIZE - 1)),
        );
        for page in kstack_pages {
            let frame = frame_alloc.allocate_frame().unwrap();
            unsafe {
                mapper.map_to(page, frame, kstack_flags, frame_alloc).unwrap().flush();    
            }
        }

        crate::println!("Copying P4");

        let p4_frame = frame_alloc.allocate_frame().unwrap();
        crate::println!("P4 frame = {:?}", p4_frame);
        unsafe {
            kernel_mapper.map_to(
                Page::containing_address(VirtAddr::new(0x4000_0000)),
                p4_frame,
                PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
                frame_alloc,
            ).unwrap().flush();
            let p4_pointer = 0x4000_0000 as *mut PageTable;
            core::ptr::copy_nonoverlapping(&page_table as *const PageTable, p4_pointer, 1);
        }

        // unsafe { asm!("mov rax, cr3", in("rax") p4_addr.as_u64()); }

        Process {
            status: ProcessStatus::Ready,
            kstack: unsafe { (KSTACK_ADDR as *const ()).as_ref().unwrap() },
            stack: unsafe { (STACK_ADDR as *const ()).as_ref().unwrap() },
            code: unsafe { (CODE_ADDR as *const ()).as_ref().unwrap() },
            page_table_frame: p4_frame,
            context: Context::new(),
        }
    }

    pub fn run(&self) -> ! {
        unreachable!()
    }

    pub fn pause(&self) {

    }
}

// TODO: impl Drop to free the stacks and co.
