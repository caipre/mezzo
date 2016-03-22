#![feature(lang_items)]
#![no_std]

extern crate rlibc;

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[lang = "panic_fmt"]
extern fn panic_fmt() -> ! {loop{}}

#[no_mangle]
pub extern fn __main__() {
    let text = b"welcome to mezzo";

    let mut bytes = [0x07; 32];
    for (i, char) in text.into_iter().enumerate() {
        bytes[i*2] = *char;
    }

    let buffer_ptr = (0xb8000 + ((80 * 2) * 12) + (80 - text.len())) as *mut _;
    unsafe { *buffer_ptr = bytes };

    loop {}
}
