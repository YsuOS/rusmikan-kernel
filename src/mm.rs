use rusmikan::MemoryMap;
use core::mem;

const MAX_PHYSICAL_MEMORY_BYTES: usize = 128 * 1024 * 1024 * 1024;
const FRAME_BYTES: usize = 4096;
const FRAME_COUNTS: usize = MAX_PHYSICAL_MEMORY_BYTES / FRAME_BYTES;
const BITS_PER_MAP_LINE: usize = 8 * mem::size_of::<usize>();

pub static mut BITMAP_MEMORY_MANAGER: BitMapMemoryManager = BitMapMemoryManager::new();

#[derive(Debug)]
pub struct BitMapMemoryManager {
    alloc_map: [usize; FRAME_COUNTS / BITS_PER_MAP_LINE],
}

impl BitMapMemoryManager {
    const fn new() -> Self {
        Self {
            alloc_map: [0; FRAME_COUNTS / BITS_PER_MAP_LINE],
        }
    }

    pub unsafe fn init(mm: &MemoryMap) {
        let mut available_end: usize = 0;
        for d in mm.descriptors() {
            let phys_start = d.phys_start as usize;
            let phys_end = d.phys_end as usize;
            if available_end < phys_start {
                let frame_id = available_end / FRAME_BYTES;
                let frame_num = (phys_start - available_end) / FRAME_BYTES;
                BITMAP_MEMORY_MANAGER.mark_allocated(frame_id, frame_num);
            }
            available_end = phys_end;
        }
    }

    fn mark_allocated(&mut self, start_frame_id: usize, frame_num: usize) {
        for i in 0..frame_num {
            let index = start_frame_id + i;
            self.set_bits(index, true);
        }
    }

    fn set_bits(&mut self, index: usize, allocated: bool) {
        let line_index = index / BITS_PER_MAP_LINE;
        let bit_index = index % BITS_PER_MAP_LINE;

        if allocated {
            self.alloc_map[line_index] |= 1 << bit_index;
        } else {
            self.alloc_map[line_index] &= !(1 << bit_index);
        }
    }
}
