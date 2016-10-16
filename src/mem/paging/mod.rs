use core::ptr::Unique;
use core::ops::{Deref, DerefMut};

use mem::{PAGE_SIZE, Frame, FrameAllocator};
use self::entry::*;
use self::table::{Table, Level4};
use self::tpage::TemporaryPage;

pub use self::mapper::Mapper;

mod entry;
mod table;
mod tpage;
mod mapper;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut TemporaryPage, f: F)
        where F: FnOnce(&mut Mapper)
    {
        {
            let backup = Frame::containing(unsafe { ::x86::shared::control_regs::cr3() } as usize);
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
            unsafe { ::x86::shared::tlb::flush_all(); }
            f(self);

            p4_table[511].set(backup, PRESENT | WRITABLE);
            unsafe { ::x86::shared::tlb::flush_all(); }
        }
        temporary_page.unmap(self);
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), PRESENT | WRITABLE)
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}

#[derive(Debug, Clone, Copy)]
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
