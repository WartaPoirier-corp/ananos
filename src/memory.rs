use x86_64::{
    structures::paging::{
        PageTable, OffsetPageTable, PhysFrame,
        Size4KiB, FrameAllocator
    },
    VirtAddr, PhysAddr,
};
use bootloader::boot_info::{MemoryRegions, MemoryRegionKind};

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

pub struct BootInfoFrameAllocator<'a> {
    memory_map: &'a MemoryRegions,
    next: usize,
}

impl<'a> BootInfoFrameAllocator<'a> {
    pub unsafe fn init(map: &'a MemoryRegions) -> BootInfoFrameAllocator<'a> {
        BootInfoFrameAllocator {
            memory_map: map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&'a self) -> impl Iterator<Item = PhysFrame> + 'a {
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

unsafe impl<'a> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

