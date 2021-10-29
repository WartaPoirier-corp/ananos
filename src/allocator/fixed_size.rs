use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;

struct Node {
    next: Option<&'static mut Node>,
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSize {
    ready: bool,
    list_heads: [Option<&'static mut Node>; BLOCK_SIZES.len()],
    fallback: super::Locked<super::linked_list::LinkedList>,
}

impl FixedSize {
    pub const fn new() -> FixedSize {
        FixedSize {
            ready: false,
            list_heads: [None, None, None, None, None, None, None, None, None],
            fallback: super::Locked::new(super::linked_list::LinkedList::new()),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.ready = true;
        self.fallback.lock().init(heap_start, heap_size)
    }

    fn list_index(layout: &Layout) -> Option<usize> {
        let required_block_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}

unsafe impl GlobalAlloc for super::Locked<FixedSize> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let list_index = FixedSize::list_index(&layout);
        if let Some(list_index) = list_index {
            match allocator.list_heads[list_index].take() {
                Some(node) => {
                    allocator.list_heads[list_index] = node.next.take();
                    node as *mut Node as *mut u8
                }
                None => {
                    let block_size = BLOCK_SIZES[list_index];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    allocator.fallback.alloc(layout)
                }
            }
        } else {
            allocator.fallback.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        let list_index = FixedSize::list_index(&layout);
        if let Some(list_index) = list_index {
            let new_node = Node {
                next: allocator.list_heads[list_index].take(),
            };

            assert!(mem::size_of::<Node>() <= BLOCK_SIZES[list_index]);
            assert!(mem::align_of::<Node>() <= BLOCK_SIZES[list_index]);
            let new_node_ptr = ptr as *mut Node;
            new_node_ptr.write(new_node);
            allocator.list_heads[list_index] = Some(&mut *new_node_ptr);
        } else {
            allocator.fallback.dealloc(ptr, layout)
        }
    }
}
