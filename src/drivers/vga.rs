use core::{convert::TryInto, fmt};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of the printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn new_line(&mut self) {
        // handle scrolling
        if self.row_position >= BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT{
                self.buffer.chars[row-1] = self.buffer.chars[row].clone();
            }
            self.clear_row(self.row_position);
        } else {
            self.row_position += 1;
        }

        // write new line
        let row = self.row_position-1;
        for col in 0..BUFFER_WIDTH {
            let character = self.buffer.chars[row][col].read();
            self.buffer.chars[row][col].write(character);
        }
        self.column_position = 0;
    }
    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.clear_char(col, row);
        }
    }
    fn clear_char(&mut self, colum: usize, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        self.buffer.chars[row][colum].write(blank);
    }
    fn draw_cursor(&self) {
        // the cursor only works if the full vga buffer already has data in it,
        // I presume because it needs the background and foreground colors. So
        // when the kernel starts we need to "clear" the vga buffer
        use x86_64::instructions::port::Port;
        let pos = self.row_position * BUFFER_WIDTH + self.column_position;
        let mut port_3d4 = Port::new(0x3d4);
        let mut port_3d5 = Port::new(0x3d5);
        unsafe{
            port_3d4.write(0xA as u8);
            port_3d5.write(0b000000 as u8); // enable cursor
            port_3d4.write(0xF as u8); // cursor location high register
            port_3d5.write((pos & 0xFF) as u8); // cursor location high register
            port_3d4.write(0xE as u8); // cursor location low register
            port_3d5.write(((pos >> 8) & 0xFF) as u8);  // cursor location low register
        }

    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        self.draw_cursor();
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_position: 0,
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn get_cursor_pos() -> (usize, usize) {
    let writer = WRITER.lock();
    return (writer.column_position.clone(), writer.row_position.clone())
}
pub fn set_cursor_pos(column: usize, row: usize) {
    let mut writer = WRITER.lock();
    writer.row_position = row;
    writer.column_position = column;
    writer.draw_cursor();
}
pub fn clear_char(col: usize, row: usize) {
    let mut writer = WRITER.lock();
    writer.clear_char(col, row);
}
pub fn clear_buffer() {
    let mut writer = WRITER.lock();
    for row in 0..BUFFER_HEIGHT {
        writer.clear_row(row);
    }
    writer.row_position = 0;
    writer.column_position = 0;
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn println_simple() {
    println!("test_println_simple output");
}
#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
