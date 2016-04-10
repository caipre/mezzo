#![no_std]

#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(range_contains)]
#![feature(unique)]

extern crate multiboot2;
extern crate rlibc;
extern crate spin;

#[macro_use]
mod vga;
use vga::*;

mod mem;
use mem::*;

#[no_mangle]
pub extern fn __main__(multiboot_info_p: usize) {
    WRITER.lock().clear();

    let boot_info = unsafe { multiboot2::load(multiboot_info_p) };

    let memory_map = boot_info.memory_map_tag().expect("no memory-map");
    println!("memory areas:");
    for area in memory_map.memory_areas() {
        println!("   start: 0x{:x}, length: 0x{:x}",
                        area.base_addr,
                        area.length);
    }

    let elf = boot_info.elf_sections_tag().expect("no elf-sections");
    println!("kernel sections:");
    for section in elf.sections() {
        println!("   addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
                        section.addr,
                        section.size,
                        section.flags);
    }

    let kernel_start = elf.sections().map(|s| s.start_address()).min().unwrap();
    let kernel_end = elf.sections().map(|s| s.end_address()).max().unwrap();

    let multiboot_start = multiboot_info_p;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);

    let mut frame_allocator = mem::AreaFrameAllocator::new(
        kernel_start, kernel_end,
        multiboot_start, multiboot_end,
        memory_map.memory_areas()
    );

    for i in 0.. {
        if let None = frame_allocator.alloc() {
            println!("allocated {} frames", i);
            break;
        }
    }


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
