use alloc::alloc::Layout;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

// We use two kinds of allocators in the kernel
//
// - A fixed size allocator, that is fast, but can only handle up to
//   a predefined amount of memory
// - A linked list allocator, slower, but only used as a fallback for
//   big allocations (since it can handle any size of allocation)

pub mod fixed_size;
pub mod linked_list;

/// Align the given address `addr` upwards to alignment `align`.
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}

/// Just a wrapper around `spin::Mutex` to be able to implement
/// traits on it (see the allocator impls).
pub struct Locked<T> {
    mutex: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(data: T) -> Self {
        Locked { mutex: spin::Mutex::new(data) }
    }


    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.mutex.lock()
    }
}

#[global_allocator]
static ALLOCATOR: Locked<fixed_size::FixedSize> = Locked::new(fixed_size::FixedSize::new());

// We use these "4444" as an easily identifiable pattern to see
// if a pointer is on the heap or the stack quicly.
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

/// Initialises a new heap.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_alloc: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_alloc)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
