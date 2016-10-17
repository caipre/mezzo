use core::ptr::Unique;
use core::ops::{Deref, DerefMut};

use multiboot2::BootInformation;

use mem::{PAGE_SIZE, Frame, FrameAllocator};
pub use self::entry::*;
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

pub fn remap_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable
    where A: FrameAllocator
{
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe }, allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.alloc().expect("no frames available");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        use self::entry::WRITABLE;
        let elf = boot_info.elf_sections_tag().expect("memory map tag missing");
        for section in elf.sections() {
            if !section.is_allocated() {
                continue;
            }
            assert!(section.addr as usize % PAGE_SIZE == 0);

            let flags = EntryFlags::from_elf_section_flags(section);
            let start = Frame::containing(section.start_address());
            let end = Frame::containing(section.end_address() - 1);
            for frame in Frame::range_inclusive(start, end) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        let vga_buffer_frame = Frame::containing(0xb8000);
        mapper.identity_map(vga_buffer_frame, WRITABLE, allocator);

        let multiboot_start = Frame::containing(boot_info.start_address());
        let multiboot_end = Frame::containing(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, PRESENT, allocator);
        }
    });

    let old_table = active_table.switch(new_table);

    let old_p4_page = Page::containing(old_table.p4_frame.start());
    active_table.unmap(old_p4_page, allocator);

    active_table
}

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

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable
    {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing(unsafe { ::x86::shared::control_regs::cr3() } as usize),
        };
        unsafe {
            ::x86::shared::control_regs::cr3_write(new_table.p4_frame.start())
        }
        old_table
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn containing(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000);
        Page { number: address / PAGE_SIZE }
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
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

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
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
