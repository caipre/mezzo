use core::ptr::Unique;

use mem::{PAGE_SIZE, Frame, FrameAllocator};
use self::entry::*;
use self::table::{Table, Level4};

mod entry;
mod table;

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

    pub fn translate(&self, vaddr: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = vaddr % PAGE_SIZE;
        self.translate_page(Page::containing(vaddr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        use self::entry::HUGE_PAGE;
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                // 1gb page?
                if let Some(frame) = p3_entry.frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        assert!(frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(
                            Frame {
                                number: frame.number +
                                        page.p2_index() * ENTRY_COUNT +
                                        page.p1_index()

                        });
                    }
                }

                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2mb page?
                    if let Some(frame) = p2_entry.frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            assert!(frame.number % ENTRY_COUNT == 0);
                            return Some(
                                Frame {
                                    number: frame.number +
                                            page.p1_index()
                            });
                        }
                    }
                }

                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].frame())
            .or_else(huge_page)
    }

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start()).is_some());
        let p1 = self.p4_mut().next_table_mut(page.p4_index())
                              .and_then(|p3| p3.next_table_mut(page.p3_index()))
                              .and_then(|p2| p2.next_table_mut(page.p2_index()))
                              .expect("mapping code does not support huge pages");
        let frame = p1[page.p1_index()].frame().unwrap();
        p1[page.p1_index()].set_unused();
        unsafe { ::x86::shared::tlb::flush(page.start()); }
        // TODO: free p1, p2, p3 table if empty
        // allocator.free(frame);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.alloc().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let page = Page::containing(frame.start());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
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

pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator
{
    let mut page_table = unsafe { ActivePageTable::new() };

    // test translation
    assert_eq!(Some(0), page_table.translate(0));
    assert_eq!(Some(4096), page_table.translate(4096));
    assert_eq!(Some(512*4096), page_table.translate(512*4096));
    assert_eq!(Some(300*512*4096), page_table.translate(300*512*4096));
    assert_eq!(None, page_table.translate(512*512*4096));
    assert_eq!(Some(512*512*4096 - 1), page_table.translate(512*512*4096 - 1));

    // test mapping
    let addr = 42 * 512 * 512 * 4096;
    let page = Page::containing(addr);
    let frame = allocator.alloc().expect("no frames available");
    assert_eq!(None, page_table.translate(addr));
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    assert_eq!(Some(0), page_table.translate(addr));
    assert!(allocator.alloc().unwrap().number > 0);

    // test unmapping
    page_table.unmap(Page::containing(addr), allocator);
    assert_eq!(None, page_table.translate(addr));



}

pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start(&self) -> usize {
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
