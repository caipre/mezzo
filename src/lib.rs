#![no_std]

#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(unique)]

extern crate rlibc;
extern crate spin;

#[macro_use]
mod vga;
use vga::*;


#[no_mangle]
pub extern fn __main__(multiboot_info_p: usize) {
    let str = "welcome to mezzo";
    WRITER.lock().move_cursor(Writer::rowalign(Align::Center),
                              Writer::colalign(Align::Center, str));
    println!("{}", str);

    loop {}
}

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    {
        WRITER.lock().set_color(ColorSpec::new(Color::LightRed, Color::Black));
        print!("\n\nkernel panic: ");
        WRITER.lock().set_color(ColorSpec::default());
        println!("{}:{}", file, line);
        println!("   {}", fmt);
    }
    loop {}
}
