#![no_std]

#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(unique)]


extern crate rlibc;

mod vga;

use core::fmt::Write;
use vga::Writer;
use vga::{AlignRow,AlignCol};

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[lang = "panic_fmt"]
extern fn panic_fmt() -> ! {loop{}}

#[no_mangle]
pub extern fn __main__() {
    let str = "welcome to mezzo";
    let mut writer = vga::Writer::new();
    writer.move_cursor(Writer::rowalign(vga::Align::Center),
                       Writer::colalign(vga::Align::Center, str));
    write!(writer, "{}", str);

    loop {}
}
