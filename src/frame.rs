use core::mem;
use lazy_static::lazy_static;
use rusmikan::MemoryMap;
use spin::Mutex;

use crate::serial_println;

pub const FRAME_BYTES: usize = 4096;
const MAX_PHYSICAL_MEMORY_BYTES: usize = 128 * 1024 * 1024 * 1024;
const FRAME_COUNTS: usize = MAX_PHYSICAL_MEMORY_BYTES / FRAME_BYTES;
const BITS_PER_MAP_LINE: usize = 8 * mem::size_of::<usize>();
const FRAME_MIN: usize = 1; // FIXME: 0 causes alloc error

pub static BITMAP_FRAME_MANAGER: Mutex<BitMapFrameManager> = Mutex::new(BitMapFrameManager::new());

#[derive(Debug)]
pub struct BitMapFrameManager {
    alloc_map: [usize; FRAME_COUNTS / BITS_PER_MAP_LINE],
    begin: usize,
    end: usize,
}

impl BitMapFrameManager {
    const fn new() -> Self {
        Self {
            alloc_map: [0; FRAME_COUNTS / BITS_PER_MAP_LINE],
            begin: FRAME_MIN,
            end: FRAME_COUNTS,
        }
    }

    pub fn init(mm: &MemoryMap) {
        let mut available_end: usize = FRAME_MIN;
        for d in mm.descriptors() {
            let phys_start = d.phys_start as usize;
            let phys_end = d.phys_end as usize;
            if available_end < phys_start {
                let frame_id = available_end / FRAME_BYTES;
                let frame_num = (phys_start - available_end) / FRAME_BYTES;
                BITMAP_FRAME_MANAGER
                    .lock()
                    .mark_allocated(frame_id, frame_num);
            }
            available_end = phys_end;
        }
        BITMAP_FRAME_MANAGER.lock().end = available_end / FRAME_BYTES;
    }

    fn mark_allocated(&mut self, start_frame_id: usize, frame_num: usize) {
        for i in 0..frame_num {
            let index = start_frame_id + i;
            self.set_bit(index, true);
        }
    }

    fn set_bit(&mut self, index: usize, allocated: bool) {
        let line_index = index / BITS_PER_MAP_LINE;
        let bit_index = index % BITS_PER_MAP_LINE;

        if allocated {
            self.alloc_map[line_index] |= 1 << bit_index;
        } else {
            self.alloc_map[line_index] &= !(1 << bit_index);
        }
    }

    fn get_bit(&mut self, index: usize) -> bool {
        let line_index = index / BITS_PER_MAP_LINE;
        let bit_index = index % BITS_PER_MAP_LINE;

        (self.alloc_map[line_index] & 1 << bit_index) != 0
    }

    pub fn allocate(&mut self, num_frames: usize) -> Option<usize> {
        let mut frame = self.begin;
        loop {
            let mut i: usize = 0;
            while i < num_frames {
                if frame + i > self.end {
                    return None;
                }
                if self.get_bit(frame + i) {
                    break;
                }
                i += 1;
            }
            if i == num_frames {
                self.mark_allocated(frame, num_frames);
                return Some(frame);
            }
            frame += i + 1;
        }
    }

    pub fn free(&mut self, frame: usize, num_frames: usize) {
        for i in 0..num_frames {
            self.set_bit(frame + i, false);
        }
    }
}

//unsafe impl FrameAllocator<Size4KiB> for BitMapFrameManager {
//    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
//        match self.allocate(1) {
//            Some(frame) => Some(PhysFrame::from_start_address(PhysAddr::new(frame as u64)).unwrap()),
//            None => None,
//        }
//    }
//}
