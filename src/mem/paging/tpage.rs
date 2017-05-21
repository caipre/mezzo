use mem::{Frame, FrameAllocator};
use super::table::{Table, Level1};
use super::{Page, ActivePageTable, VirtualAddress};

pub struct TemporaryPage {
    page: Page,
    allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage
        where A: FrameAllocator
    {
        TemporaryPage {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        use super::entry::WRITABLE;

        assert!(active_table.translate_page(self.page).is_none());
        active_table.map_to(self.page, frame, WRITABLE, &mut self.allocator);
        self.page.start()
    }

    pub fn map_table_frame(&mut self,
                           frame: Frame,
                           active_table: &mut ActivePageTable)
                           -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }

    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator);
    }
}

struct TinyAllocator([Option<Frame>; 3]);

impl TinyAllocator {
    fn new<A>(allocator: &mut A) -> TinyAllocator
        where A: FrameAllocator
    {
        let mut alloc = || allocator.alloc();
        let frames = [alloc(), alloc(), alloc()];
        TinyAllocator(frames)
    }
}

impl FrameAllocator for TinyAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        for frame in &mut self.0 {
            if frame.is_some() {
                return frame.take();
            }
        }
        None
    }

    fn free(&mut self, frame: Frame) {
        for frame_ in &mut self.0 {
            if frame_.is_none() {
                *frame_ = Some(frame);
                return;
            }
        }
        panic!("TinyAllocator only holds 3 frames!");
    }
}
