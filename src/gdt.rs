use x86_64::VirtAddr;
use x86_64::structures::{
    tss::TaskStateSegment,
    gdt::{
        GlobalDescriptorTable, Descriptor,
        SegmentSelector,
    }
};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize]= {
            const STACK_SIZE: usize = 4096;
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
        (gdt, Selectors {
            code_selector: cs_sel,
            tss_selector: tss_sel,
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
