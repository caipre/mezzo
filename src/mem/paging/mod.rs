use core::ptr::Unique;

use mem::PAGE_SIZE;
use mem::paging::table::{Table, Level4};

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new(table::P4),
        }
    }

    pub fn translate(vaddr: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = vaddr % PAGE_SIZE;
        translate_page(Page::containing(vaddr))
            .map(|frame| frame.number * PAGE_SIZE + offset);
    }

    fn translate_page(page: Page) -> Option<Frame> {
        use self::entry::HUGE_PAGE;
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            panic!("huge pages not implemented")
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].frame())
        .or_else(huge_page)
    }

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator {
        assert!(self.translate(page.start_address()).is_some());
        let p1 = self.p4_mut().next_table_mut(page.p4_index())
                              .and_then(|p3| p3.next_table_mut(page.p3_index()))
                              .and_then(|p2| p2.next_table_mut(page.p2_index()))
                              .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].frame().unwrap();
        p1[page.p1_index()].set_unused();
        // TODO: free p1, p2, p3 table if empty
        allocator.free(frame);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator {
        let frame = allocator.alloc().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator {
        let page = Page::containing(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn map_to<A>(page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator {
        let p4 = self.p4_mut();
        let mut p3 = p4.next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags|PRESENT);
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.get() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.get_mut() }
    }

}

pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address > 0xffff_8000_0000_0000);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}
