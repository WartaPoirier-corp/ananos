use super::align_up;
use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;

/// A node of a linked list, that corresponds to a free
/// heap area in the context of this allocator.
struct Node {
    size: usize,
    next: Option<&'static mut Node>,
}

impl Node {
    /// Create a new free area of a given size.
    ///
    /// It doesn't actually free anything in memory, and it
    /// is not automatically added to the list of free areas,
    /// it is just a constructor function.
    const fn new(size: usize) -> Node {
        Node { next: None, size }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// The actual allocator
pub struct LinkedList {
    head: Node,
}

impl LinkedList {
    pub const fn new() -> LinkedList {
        LinkedList { head: Node::new(0) }
    }

    /// Initializes the allocator with a memory region on which it has the right to work.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    pub unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // Makes sure we can store our Node in the free region
        assert_eq!(align_up(addr, mem::align_of::<Node>()), addr);
        assert!(size >= mem::size_of::<Node>());

        let mut node = Node::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut Node;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    /// Finds a free region
    ///
    /// Returns the region and its size.
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut Node, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut reg) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&reg, size, align) {
                let next = reg.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }

        None
    }

    fn alloc_from_region(region: &Node, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(()); // The region was too small
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<Node>() {
            return Err(()); // We don't have enough space to store our metadata
        }

        Ok(alloc_start)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::size_of::<Node>())
            .expect("Adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<Node>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for super::Locked<LinkedList> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedList::size_align(layout);
        let mut allocator = self.lock();

        if let Some((reg, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = match alloc_start.checked_add(size) {
                Some(x) => x,
                _ => return ptr::null_mut(),
            };

            let excess_size = reg.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }

            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedList::size_align(layout);

        self.lock().add_free_region(ptr as usize, size);
    }
}
