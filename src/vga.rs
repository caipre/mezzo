// use core::default::Default;    // not useful without const trait fns

use core::cmp::{min,max};
use core::fmt;
use core::ops::{Index, IndexMut};
use core::option::Option;
use core::ptr::Unique;

use spin::Mutex;

const BUFFER_ROWS: usize = 25;
const BUFFER_COLS: usize = 80;

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black      = 0x0,    DarkGray   = 0x8,
    Blue       = 0x1,    LightBlue  = 0x9,
    Green      = 0x2,    LightGreen = 0xa,
    Cyan       = 0x3,    LightCyan  = 0xb,
    Red        = 0x4,    LightRed   = 0xc,
    Magenta    = 0x5,    Pink       = 0xd,
    Brown      = 0x6,    Yellow     = 0xe,
    LightGray  = 0x7,    White      = 0xf,
}

#[derive(Copy, Clone)]
pub struct ColorSpec(u8);

impl ColorSpec {
    pub const fn new(foreground: Color, background: Color) -> ColorSpec {
        ColorSpec((background as u8) << 4 | (foreground as u8))
    }

    pub const fn default() -> ColorSpec {
        ColorSpec::new(Color::LightGray, Color::Black)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VgaChar {
    char: u8,
    spec: ColorSpec,
}

impl VgaChar {
    const fn default() -> VgaChar {
        VgaChar { char: b' ', spec: ColorSpec::default() }
    }
}

#[derive(Copy, Clone)]
struct VgaBuffer {
    chars: [[VgaChar; BUFFER_COLS]; BUFFER_ROWS],
}

impl VgaBuffer {
    const ADDRESS: usize = 0xb8000;

    const fn buffer() -> *mut VgaBuffer {
        VgaBuffer::ADDRESS as *mut VgaBuffer
    }
}

impl Index<usize> for VgaBuffer {
    type Output = [VgaChar; BUFFER_COLS];
    fn index<'a>(&'a self, index: usize) -> &Self::Output {
        &self.chars[index]
    }
}

impl IndexMut<usize> for VgaBuffer {
    fn index_mut<'a>(&'a mut self, index: usize) -> &mut Self::Output {
        &mut self.chars[index]
    }
}

#[allow(dead_code)]
pub enum Align {
    Top,
    Center,
    Right,
    Bottom,
    Left,
}

pub trait AlignRow {
    fn rowalign(align: Align) -> usize;
}

pub trait AlignCol {
    fn colalign(align: Align, str: &str) -> usize;
}

pub struct Writer {
    row: usize, col: usize,
    color_spec: ColorSpec,
    buffer: Unique<VgaBuffer>,
}

pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new());

impl Writer {
    const fn new() -> Writer {
        Writer {
            row: 0, col: 0,
            color_spec: ColorSpec::default(),
            buffer: unsafe { Unique::new(VgaBuffer::buffer()) },
        }
    }

    pub fn move_cursor(&mut self, row: usize, col: usize) {
        self.row = min(row, BUFFER_ROWS - 1);
        self.col = min(col, BUFFER_COLS - 1);
    }

    pub fn clear(&mut self) {
        for row in 0..(BUFFER_ROWS - 1) {
            self.row = row;
            self.clear_row();
        }
        self.move_cursor(0, 0);
    }

    pub fn set_color(&mut self, spec: ColorSpec) {
        self.color_spec = spec;
    }

    fn write_byte(&mut self, byte: u8, spec: Option<ColorSpec>) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                self.buffer().chars[self.row][self.col] = VgaChar {
                    char: byte,
                    spec: spec.unwrap_or(self.color_spec),
                };
                self.col += 1;
                if self.col >= BUFFER_COLS {
                    self.new_line();
                }
            }
        }
    }

    fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.get_mut() }
    }

    fn new_line(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row == BUFFER_ROWS - 1 {
            self.scroll_up();
            self.row -= 1;
            self.clear_row();
        }
    }

    fn scroll_up(&mut self) {
        for row in 0..(BUFFER_ROWS - 1) {
            self.buffer().chars[row] = self.buffer().chars[row + 1];
        }
    }

    fn clear_row(&mut self) {
        self.buffer().chars[self.row] = [VgaChar::default(); BUFFER_COLS];
    }
}

impl AlignRow for Writer {
    fn rowalign(align: Align) -> usize {
        match align {
            Align::Top => 0,
            Align::Center => (BUFFER_ROWS - 1) / 2,
            Align::Bottom => BUFFER_ROWS - 1,
            _ => panic!(),
        }
    }
}

impl AlignCol for Writer {
    fn colalign(align: Align, str: &str) -> usize {
        match align {
            Align::Left => 0,
            Align::Center => ((BUFFER_COLS - 1) - str.len()) / 2,
            Align::Right => max(0, (BUFFER_COLS - 1) - str.len()),
            _ => panic!(),
        }
    }
}

impl ::core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
          self.write_byte(byte, None)
        }
        Ok(())
    }
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        $crate::vga::WRITER.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

pub unsafe fn kerror(fmt: fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = Writer {
        col: 0, row: 0,
        color_spec: ColorSpec::new(Color::LightRed, Color::Black),
        buffer: Unique::new(0xb8000 as *mut _),
    };
    writer.write_str("\n\nkernel panic: ").unwrap();
    writer.set_color(ColorSpec::default());
    writer.write_fmt(fmt).unwrap();
}
