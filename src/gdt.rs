use x86_64::VirtAddr;
use x86_64::structures::{
    tss::TaskStateSegment,
    gdt::{
        GlobalDescriptorTable, Descriptor,
        SegmentSelector,
    },
    paging::{Mapper, FrameAllocator, Size4KiB},
};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 2;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 8192;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 8192;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.interrupt_stack_table[GENERAL_PROTECTION_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 8192;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let cs_sel = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_sel = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let user_data_sel = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_sel = gdt.add_entry(Descriptor::user_code_segment());
        (gdt, Selectors {
            code_selector: cs_sel,
            tss_selector: tss_sel,
            user_code_selector: user_code_sel,
            user_data_selector: user_data_sel,
        })
    };
}

pub fn init() {
    use x86_64::instructions::{
        segmentation::set_cs,
        tables::load_tss,
    };

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

/// A function to enter usermode
pub fn do_context_switch(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_alloc: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::instructions::interrupts;
    use x86_64::structures::paging::{
        Page, PageTableFlags,
    };

    unsafe {
        interrupts::disable();

        let data_sel = GDT.1.user_data_selector.0;
        let code_sel = GDT.1.user_code_selector.0;

        // Map a user stack and some space for code
        const STACK: u64 = 0x600_000;
        const CODE: u64 = 0x400_000;

        let frame = frame_alloc.allocate_frame().unwrap();
        let page = Page::containing_address(VirtAddr::new(STACK));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        mapper.map_to(page, frame, flags, frame_alloc).unwrap().flush();

        let frame = frame_alloc.allocate_frame().unwrap();
        let page = Page::containing_address(VirtAddr::new(CODE));
        mapper.map_to(page, frame, flags, frame_alloc).unwrap().flush();

        let code = CODE as *mut u8;
        // nop
        core::ptr::write(code, 0x90);
        // mov ax, 0x00
        core::ptr::write(code.add(1), 0x66);
        core::ptr::write(code.add(2), 0xb8);
        core::ptr::write(code.add(3), 0x00);
        core::ptr::write(code.add(4), 0x00);
        // mov bx, 0xFF
        core::ptr::write(code.add(5), 0x66);
        core::ptr::write(code.add(6), 0xbb);
        core::ptr::write(code.add(7), 0xff);
        core::ptr::write(code.add(8), 0x00);
        // int 0x80 (system call)
        core::ptr::write(code.add(9), 0xcd);
        core::ptr::write(code.add(10), 0x80);

        asm!(
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",

            "push rax",
            "push rbx",
            
            "pushf",
            "pop rax",
            "or rax, 0x200",
            "push rax",
            
            "push rcx",
            "push rdx",
            "iretq",
            in("rax") data_sel,
            in("rbx") STACK,
            in("rcx") code_sel,
            in("rdx") CODE,
        );
        unreachable!();
    }
}
