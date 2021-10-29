use crate::gdt::GDT;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{FrameAllocator, Mapper, Size4KiB};
use x86_64::structures::paging::{Page, PageTableFlags};
use x86_64::VirtAddr;

// TODO: use virtual memory better, i.e don't map all
// processes in the same page table directory
static STACK_ADDR: AtomicU64 = AtomicU64::new(0x600_000);
static CODE_ADDR: AtomicU64 = AtomicU64::new(0x400_000);

lazy_static::lazy_static! {
    static ref PROCESSES: spin::RwLock<Vec<u64>> = spin::RwLock::new(
        Vec::with_capacity(8),
    );

    // TODO: one per AP
    // TODO: process delta queue instead of this (the head being the current proc)
    static ref CURRENT_PID: spin::RwLock<Option<PId>> = spin::RwLock::new(None);
}

#[derive(Clone, Copy, Debug)]
pub struct PId(usize);

impl PId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

pub fn current() -> Option<PId> {
    CURRENT_PID.try_read().and_then(|x| *x)
}

pub fn spawn(proc: Process<'_>) -> Option<PId> {
    if let Some(ref mut proc_list) = PROCESSES.try_write() {
        let boxed = Box::new(proc);
        let ptr = Box::leak(boxed) as *const _ as u64;
        proc_list.push(ptr);
        Some(PId(proc_list.len() - 1))
    } else {
        None
    }
}

pub fn switch_to(pid: PId) {
    if let Some(proc_list) = PROCESSES.try_read() {
        if let Some(mut curr_pid) = CURRENT_PID.try_write() {
            *curr_pid = Some(pid);
        }

        let proc = unsafe { &*(proc_list[pid.0] as *mut Process<'static>) };
        proc.switch();
    }
}

pub fn get_mut<'a>(pid: PId) -> Option<&'a mut Process<'a>> {
    if let Some(proc_list) = PROCESSES.try_read() {
        let proc = unsafe { &mut *(proc_list[pid.0] as *mut Process<'a>) };
        Some(proc)
    } else {
        None
    }
}

pub struct Stream<'a> {
    iter: adb::TypeIterator<'a, alloc::vec::Vec<u8>>,
}

impl<'a> Stream<'a> {
    pub fn ty(&self) -> adb::TypeId {
        self.iter.ty().id
    }
}

#[derive(Default)]
struct State {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rsp: u64,
    rbp: u64,
}

pub struct Process<'a> {
    stack_addr: u64,
    code_addr: u64,
    pub streams: alloc::vec::Vec<Stream<'a>>,
    state: State,
}

impl<'a> Process<'a> {
    pub fn create(
        mapper: &mut impl Mapper<Size4KiB>,
        frame_alloc: &mut impl FrameAllocator<Size4KiB>,
        asm: &[u8],
    ) -> Process<'a> {
        const PAGE_SIZE: u64 = 1024 * 4;
        let frame = frame_alloc.allocate_frame().unwrap();
        let stack = STACK_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let page = Page::containing_address(VirtAddr::new(stack));
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_alloc)
                .unwrap()
                .flush();
        }

        let frame = frame_alloc.allocate_frame().unwrap();
        let code = CODE_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let page = Page::containing_address(VirtAddr::new(code));
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_alloc)
                .unwrap()
                .flush();
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
            streams: alloc::vec::Vec::with_capacity(8),
            state: State::default(),
        }
    }

    pub fn open_stream(&mut self, db: &'a mut adb::Db<alloc::vec::Vec<u8>>, ty: adb::TypeId) {
        self.streams.push(Stream {
            iter: db.iter_type(ty),
        });
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
