
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt::Write;

#[macro_export] // makes macro available everywhere in crate
macro_rules! print { // expands to _print function
    // $crate variable ensures it macro works outside our crate
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n")); // "println!()" should just print newline
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*))); // just exapnd to print! and add newline
}

// needs to be public bcs macros need to be able to call it from outside the module
#[doc(hidden)] // but its considered private, so this attribute hides it from the generated docs
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

lazy_static! { // lazy static needed here, as Rust’s const evaluator is not able to convert raw pointers to references at compile time
    static ref WRITER: Mutex<Writer> = Mutex::new(Writer { // Mutex needed so it can be mutable
        column_position: 0,
        color_code: ColorCode::new(Color::Magenta, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}, // VGA buffers memmory address is 0xb8000
    });
}

struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte), // int UTF-8 the individual bytes of multi-byte values are never valid ASCII
                // not part of printable ASCII range
                _ => self.write_byte(0xfe), // 0xfe is ■
            }
        }
    }
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; // always write to bottom row
                let col = self.column_position;
                let color_code = self.color_code;
                
                // using Volatile write to directly write data to VGA buffer
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // move each character one row up
        for row in 1..BUFFER_HEIGHT { // omiting 0th row, it'll be shifted off screen (overwritten in practice)
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1); // clear row at the bottom
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// trait implemented to allow usage of write! macros
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

 // can only be used on a struct that has a single non-zero-sized field
#[repr(transparent)] // guarantees the layout to be the same as that one field
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // forces field ordering as defined in struct
struct ScreenChar { // first byte represents the character, second byte defines how it should be displayed
    ascii_character: u8,
    color_code: ColorCode,
}

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

    // MSB is used as "bright bit" - repurposed as blink bitfor background colors 
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

#[test_case]
fn test_println_simple() {
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
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}