#![no_std]

#![feature(alloc)]
#![feature(asm)]
#![feature(associated_consts)]
#![feature(collections)]
#![feature(const_fn)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(range_contains)]
#![feature(unique)]

extern crate alloc;
#[macro_use]
extern crate collections;

#[macro_use]
extern crate bitflags;
extern crate bit_field;
#[macro_use]
extern crate lazy_static;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
#[macro_use]
extern crate x86;

extern crate holealloc;

use alloc::boxed::Box;

#[macro_use]
mod vga;
use vga::*;

mod mem;
use mem::*;

mod int;

#[no_mangle]
pub extern fn __main__(multiboot_info_p: usize) {
    WRITER.lock().clear();

    let boot_info = unsafe {
        multiboot2::load(multiboot_info_p)
    };

    enable_nxe_bit();
    enable_write_protect_bit();

    mem::init(boot_info);
    int::init();

    unsafe { int!(3) };
    unsafe { *(0xdeadbeef as *mut u64) = 42 };
    println!("it did not crash");

    loop {}
}

fn enable_nxe_bit() {
    use ::x86::shared::msr::{IA32_EFER, rdmsr, wrmsr};
    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

fn enable_write_protect_bit() {
    use ::x86::shared::control_regs::{CR0_WRITE_PROTECT, cr0, cr0_write};
    unsafe {
        cr0_write(cr0() | CR0_WRITE_PROTECT);
    }
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    unsafe {
        vga::kerror(format_args!("{}:{}\n   {}", file, line, fmt));
    }
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
