use bit_field::BitField;
use x86::shared::segmentation::{self, SegmentSelector};

pub struct Idt([Entry; 16]);

impl Idt {
    pub fn new() -> Idt {
        Idt([Entry::missing(); 16])
    }

    pub fn load(&'static self) {
        use core::mem::size_of;
        use x86::shared::dtables::{DescriptorTablePointer, lidt};
        let ptr = DescriptorTablePointer {
            base: self as *const _ as *const _,
            limit: (size_of::<Idt>() - 1) as u16,
        };
        unsafe { lidt(&ptr) };
    }

    pub fn set_handler(&mut self, entry: u8, handler: HandlerFunc) -> &mut EntryOptions {
        self.0[entry as usize] = Entry::new(segmentation::cs(), handler);
        &mut self.0[entry as usize].options
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Entry {
    low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    mid: u16,
    high: u32,
    reserved: u32,
}

impl Entry {
    fn new(gdt_selector: SegmentSelector, handler: HandlerFunc) -> Entry {
        let pointer = handler as u64;
        Entry {
            gdt_selector: gdt_selector,
            low: pointer as u16,
            mid: (pointer >> 16) as u16,
            high: (pointer >> 32) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }

    fn missing() -> Entry {
        use x86::shared::PrivilegeLevel;
        Entry {
            gdt_selector: SegmentSelector::new(0, PrivilegeLevel::Ring0),
            low: 0,
            mid: 0,
            high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn minimal() -> EntryOptions {
        let mut options = 0;
        options.set_range(9..12, 0b111);
        EntryOptions(options)
    }

    fn new() -> EntryOptions {
        let mut options = EntryOptions::minimal();
        options.present(true).interruptible(false);
        options
    }

    pub fn present(&mut self, present: bool) -> &mut EntryOptions {
        self.0.set_bit(15, present);
        self
    }

    pub fn interruptible(&mut self, interrupts: bool) -> &mut EntryOptions {
        self.0.set_bit(8, interrupts);
        self
    }

//    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut EntryOptions {
//        self.0.set_range(13..15, dpl);
//        self
//    }
//
//    pub fn set_stack_index(&mut self, index: u16) -> &mut EntryOptions {
//        self.0.set_range(0..3, index);
//        self
//    }
}

pub type HandlerFunc = extern "C" fn() -> !;
