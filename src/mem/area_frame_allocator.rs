use core::ops::Range;

use mem::{Frame, FrameAllocator};
use multiboot2::{MemoryArea, MemoryAreaIter};

pub struct AreaFrameAllocator {
    area: Option<&'static MemoryArea>,
    areas: MemoryAreaIter,
    next: Frame,
    kernel: Range<Frame>,
    multiboot: Range<Frame>,
}

impl FrameAllocator for AreaFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        if let Some(area) = self.area {
            let frame = Frame { number: self.next.number };

            let last_frame = {
                let last_address = area.base_addr + area.length - 1;
                Frame::containing(last_address as usize)
            };

            if frame > last_frame {
                self.select_next_area();
                return self.alloc();
            } else {
                self.next.number += 1;
                return Some(frame);
            }
        }
        None
    }

    fn free(&mut self, _frame: Frame) {
        unimplemented!()
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize,
               kernel_end: usize,
               multiboot_start: usize,
               multiboot_end: usize,
               memory_areas: MemoryAreaIter)
               -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            area: None,
            areas: memory_areas,
            next: Frame::containing(0),
            kernel: Range {
                start: Frame::containing(kernel_start),
                end: Frame::containing(kernel_end),
            },
            multiboot: Range {
                start: Frame::containing(multiboot_start),
                end: Frame::containing(multiboot_end),
            },
        };
        allocator.select_next_area();
        allocator
    }

    fn select_next_area(&mut self) {
        self.area = self.areas
            .clone()
            .filter(|area| {
                let last_address = area.base_addr + area.length - 1;
                let frame = Frame::containing(last_address as usize);

                frame >= self.next &&
                !(self.kernel.start.number..(self.kernel.end.number)).contains(frame.number) &&
                !(self.multiboot.start.number..(self.multiboot.end.number))
                     .contains(frame.number)
            })
            .min_by_key(|area| area.base_addr);

        if let Some(area) = self.area {
            let first_frame = Frame::containing(area.base_addr as usize);
            if self.next < first_frame {
                self.next = first_frame;
            }
        }
    }
}
