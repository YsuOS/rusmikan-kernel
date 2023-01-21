use core::{alloc::{Layout, GlobalAlloc}, mem};
use x86_64::VirtAddr;

use crate::{serial_println, frame::{FRAME_BYTES, BITMAP_FRAME_MANAGER}, panic};
use core::ptr;

#[global_allocator]
pub static ALLOCATOR: Locked<KernelAllocator> = Locked::new(KernelAllocator::new());

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

#[derive(Debug)]
struct ListNode {
    next: Option<&'static mut ListNode>,
}

#[derive(Debug)]
pub struct KernelAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
}

impl KernelAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        KernelAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
        }
    }

    unsafe fn allocate_frame_for_block(&mut self, index: usize) -> *mut u8 {
        let block_size = BLOCK_SIZES[index];
        let num_blocks_per_frame = FRAME_BYTES / block_size;
    
        let ptr: *mut u8 = match BITMAP_FRAME_MANAGER.allocate(1) {
            Some(frame) => VirtAddr::new((frame*FRAME_BYTES) as u64).as_u64() as *mut u8,
            None => return ptr::null_mut(),
        };
        for i in (0..num_blocks_per_frame).rev() {
            let current = ptr.add(i * block_size);
            let next = current.add(block_size) as *mut ListNode;
            let new_node = ListNode {
                next: self.list_heads[index].take(),
            };
            next.write(new_node);
            self.list_heads[index] = Some(&mut *next);
        }
        ptr
    }
}

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

unsafe impl GlobalAlloc for Locked<KernelAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                serial_println!("DEBUG: alloc size: {:?}", BLOCK_SIZES[index]);
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => { 
                        let ptr = allocator.allocate_frame_for_block(index);
                        ptr as *mut ListNode as *mut u8
                    }
                }},
            None => { 
                // TODO: support > 4096 size allocation
                serial_println!("No index. allocate frame {:?}", layout.size());
                if layout.size() == FRAME_BYTES {
                    match BITMAP_FRAME_MANAGER.allocate(layout.size() / FRAME_BYTES) {
                        Some(frame) => VirtAddr::new((frame*FRAME_BYTES) as u64).as_u64() as *mut u8,
                        None => panic!("Out Of Memory"),
                    }
                } else {
                    panic!("heap allocation over 4 KiB is not supported!");
                }
            },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };

                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            },
            None => {
                // TODO: support > 4096 size allocation
                BITMAP_FRAME_MANAGER.free(ptr as usize / FRAME_BYTES, 1);
            },
        }
    }

}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}
