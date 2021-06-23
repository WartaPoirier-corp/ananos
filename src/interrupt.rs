use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame,
    PageFaultErrorCode
};
use crate::println;
use crate::gdt;
use spin;
use pic8259_simple::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static::lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.set_handler_fn(page_fault_handler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
            idt.general_protection_fault.set_handler_fn(gp_handler)
                .set_stack_index(gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);

            idt.slice_mut(0x80..0x81)[0]
                .set_handler_fn(syscall)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        }
        idt.stack_segment_fault.set_handler_fn(ss_fault_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present);

        // PIC interrupts
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn syscall(
    _stack: InterruptStackFrame,
) {
    let code: usize;
    let arg1: usize;
    unsafe {
        asm!(
            "",
            out("rax") code,
            out("rbx") arg1,
        );
    }
    println!("system call: {} {}", code, arg1);
    match code {
        0 => {
            let mut fb = crate::FB.lock();
            if let Some(ref mut fb) = *fb {
                let buff = unsafe {
                    core::slice::from_raw_parts_mut(
                        fb.0 as *mut u8,
                        fb.1
                    )
                }; 
                for byte in buff {
                    *byte = arg1 as u8;
                }
            }
        },
        _ => {},
    }
    loop {}
}

extern "x86-interrupt" fn segment_not_present(
    stack: InterruptStackFrame,
    code: u64,
) {
    let ip = stack.instruction_pointer.as_ptr();
    let inst: [u8; 8] = unsafe { core::ptr::read(ip) };
    println!("Code: {:?}", inst);
    println!("SEGMENT NOT PRESENT ({:b}) at {:?}", code, ip);
    loop {}
}
extern "x86-interrupt" fn page_fault_handler(
    stack: InterruptStackFrame,
    error_code: PageFaultErrorCode
) {
    println!("PAGE FAULT");
    let ip = stack.instruction_pointer.as_ptr();
    let inst: [u8; 8] = unsafe { core::ptr::read(ip) };
    println!("Code: {:?}", inst);
    println!("{:#?}\n{:#?}", stack, error_code);
    loop {}
}

extern "x86-interrupt" fn gp_handler(
    stack: InterruptStackFrame,
    code: u64,
) {
    let ip = stack.instruction_pointer.as_ptr();
    let inst: [u8; 8] = unsafe { core::ptr::read(ip) };
    println!("Code: {:?}", inst);
    let sp = stack.stack_pointer.as_ptr();
    let st: [u64; 32] = unsafe { core::ptr::read(sp) };
    crate::println!("----------\nStack at {:p}", ip);
    for s in st.iter() { crate::println!("{:#018x} ({:#065b})", s, s); }
    println!("GENERAL PROTECTION FAULT ({:#x} = {:#b}) at {:?}", code, code, ip);
    println!("{:#?}", stack);
    loop {}
}

extern "x86-interrupt" fn ss_fault_handler(
    _stack: InterruptStackFrame,
    code: u64,
) {
    println!("STACK SEGMENT FAULT ({})", code);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack: InterruptStackFrame
) {
    use x86_64::instructions::port::Port;


    let mut port = Port::new(0x60); // Keyboard I/O port
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack: InterruptStackFrame,
) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn breakpoint_handler(
    stack: InterruptStackFrame
) {
    // FIXME: it looks like this handler causes a page fault
    crate::println!("BREAKPOINT: {:#?}", stack);
    let ip = stack.instruction_pointer.as_ptr();
    let inst: [u8; 8] = unsafe { core::ptr::read(ip) };
    crate::println!("-------------------\nCode: {:?}", inst);
    let sp = stack.stack_pointer.as_ptr();
    let st: [u64; 32] = unsafe { core::ptr::read(sp) };
    crate::println!("Stack at {:p}", ip);
    for s in st.iter() { crate::println!("{:#018x} ({:#065b})", s, s); }
}

extern "x86-interrupt" fn double_fault_handler(
    stack: InterruptStackFrame,
    error_code: u64
) -> ! {
    let ip = stack.instruction_pointer.as_ptr();
    let ip: [u8; 8] = unsafe { core::ptr::read(ip) };
    println!("Code: {:?}", ip);
    panic!("DOUBLE FAULT (code : {}) : {:#?}", error_code, stack);
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint() {
        x86_64::instructions::interrupts::int3();
    }
}

