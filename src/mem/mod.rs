use core::option::Option;


mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::paging::test_paging;

use self::paging::PhysicalAddress;

pub const PAGE_SIZE: usize = 4096;

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
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn free(&mut self, frame: Frame);
}
