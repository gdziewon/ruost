use core::{alloc::{GlobalAlloc, Layout}, ptr::{self, NonNull}};
use core::mem;
use super::Locked;

// sizes must be power of 2 bcs they are also used as alignment (which must be powers of 2)
// smallest is 8 because each needs to fit 64-bit pointer
// for allocations greater then 2048 fallback to linked list allocator
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

// no need for size field since size will be embedded in each linked list
struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap, // use heap from crate, bcs our implementation doesnt merge blocks
}

// linked list for each size of block
// it comes with big performence boost over standard linked list allocator
// when using powers of 2 as blocks sizes, 3/4 of memory is wasted on average tho
impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) { // index of needed block size
            Some(index) => { // found block size
                match allocator.list_heads[index].take() { // get first node
                    Some(node) => { // node found
                        allocator.list_heads[index] = node.next.take(); // switch heads
                        node as *mut ListNode as *mut u8
                    }
                    None => { // list is empty, create blocks
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align)
                            .unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            } // block size not found, fallback to linked list
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that block has size and alignemnt required for storing node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr); // set it as head
            }
            None => { // no corresponding block size found, so it was allocated via fallback allocator
                let ptr = NonNull::new(ptr).unwrap();
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        }
    }
}


// choose appropriate block size for the given layout
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}