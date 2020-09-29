use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use x86_64::instructions::interrupts;
use crate::process::Process;
use crate::gdt;

lazy_static::lazy_static! {
    pub static ref SCHEDULER: Mutex<Arc<Scheduler<'static>>> = Mutex::new(Arc::new(Scheduler {
        processes: Vec::with_capacity(16),
        current: None,
    }));
}

pub struct Scheduler<'a> {
    processes: Vec<Process<'a>>,
    current: Option<usize>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        Self {
            processes: Vec::with_capacity(16),
            current: None,
        }
    }

    pub fn spawn<'b>(&'b mut self, proc: Process<'a>) {
        self.processes.push(proc);
    }

    pub fn next<'b>(&'b mut self) {
        if self.processes.is_empty() {
            return;
        }

        let next_id = 0;
        self.current = Some(next_id);
        let process = &self.processes[next_id];

        interrupts::disable();
        let sel = gdt::get_selectors();
        let data_sel = sel.user_data_selector.0;
        let code_sel = sel.user_code_selector.0;
        const STACK: u64 = 0x1000_0000;
        const CODE: u64 = 0x2000_0000;
        let p4_addr = process.page_table_frame.start_address().as_u64();

        unsafe {
            asm!(
                "mov cr3, r15",
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
                in("r15") p4_addr,
            );    
        }
    }
}

