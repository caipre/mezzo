mod area_frame_allocator;
mod paging;

use multiboot2::BootInformation;

pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::paging::test_paging;

use self::paging::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

pub fn init(boot_info: &BootInformation) {
    assert_has_not_been_called!();

    let memory_map = boot_info.memory_map_tag().expect("no memory-map");
    let elf = boot_info.elf_sections_tag().expect("no elf-sections");

    let kernel_start = elf.sections()
        .filter(|s| s.is_allocated()).map(|s| s.start_address()).min().unwrap();
    let kernel_end = elf.sections()
        .filter(|s| s.is_allocated()).map(|s| s.end_address()).max().unwrap();

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start, kernel_end,
        boot_info.start_address(), boot_info.end_address(),
        memory_map.memory_areas()
    );

    let mut active_table = paging::remap_kernel(&mut frame_allocator, boot_info);

    use self::paging::Page;
    use bumpalloc::{HEAP_START, HEAP_SIZE};

    let heap_start_page = Page::containing(HEAP_START);
    let heap_end_page = Page::containing(HEAP_START + HEAP_SIZE - 1);
    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, paging::WRITABLE, &mut frame_allocator);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    fn start(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    fn containing(address: usize) -> Frame {
        Frame { number: address / PAGE_SIZE }
    }

    fn clone(&self) -> Frame {
        Frame { number: self.number }
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }

}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn free(&mut self, frame: Frame);
}
