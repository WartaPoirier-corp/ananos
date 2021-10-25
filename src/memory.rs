use alloc::rc::Rc;
use core::ptr::NonNull;
use acpi::PhysicalMapping;
use x86_64::{PhysAddr, VirtAddr, structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PageTableFlags, PhysFrame, Size4KiB, page::PageRangeInclusive, Page, Mapper}};
use bootloader::boot_info::{MemoryRegions, MemoryRegion, MemoryRegionKind};
use spin::Mutex;

pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_page_level_4_table(phys_mem_offset);
    OffsetPageTable::new(l4_table, phys_mem_offset)
}

unsafe fn active_page_level_4_table(
    physical_mem_offset: VirtAddr
) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_mem_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static mut [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(map: &'static mut [MemoryRegion]) -> Self {
        BootInfoFrameAllocator {
            memory_map: map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.start..r.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

const ACPI_OFFSET: usize = 0x1000_0000_0000;

#[derive(Clone)]
pub struct AcpiHandler {
    mapper: Rc<Mutex<OffsetPageTable<'static>>>,
    frame_allocator: Rc<Mutex<BootInfoFrameAllocator>>,
}

impl AcpiHandler {
    pub fn new(mapper: OffsetPageTable<'static>, frame_allocator: BootInfoFrameAllocator) -> Self {
        Self {
            mapper: Rc::new(Mutex::new(mapper)),
            frame_allocator: Rc::new(Mutex::new(frame_allocator)),
        }
    }
}

impl acpi::AcpiHandler for AcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> acpi::PhysicalMapping<Self, T> {
        let start = Page::containing_address(VirtAddr::new((ACPI_OFFSET + physical_address) as u64));
        let end = Page::containing_address(VirtAddr::new((ACPI_OFFSET + physical_address + size) as u64));
        let range = Page::range_inclusive(start, end);
        let mut allocator = self.frame_allocator.lock();
        let mut mapper = self.mapper.lock();
        for page in range {
            let frame = allocator
                .allocate_frame()
                .unwrap();
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            mapper.map_to(page, frame, flags, &mut *allocator).unwrap().flush();
        }
        PhysicalMapping::new(
            physical_address,
            NonNull::new((ACPI_OFFSET + physical_address) as *mut _).unwrap(),
            size,
            (end.start_address() + end.size() - start.start_address()) as usize,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        
    }
}
