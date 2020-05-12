#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct VgaChar {
    ascii: u8,
    color: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize= 25;

#[repr(transparent)]
// We wrap VgaChar in a Volatile so that rustc doesn't optimize writes away
pub struct Buffer([[volatile::Volatile<VgaChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]);

pub struct Writer {
    col_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new(color: ColorCode) -> Writer {
        Writer {
            col_pos: 0,
            color_code: color,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.col_pos;
                let color_code = self.color_code;
                self.buffer.0[row][col].write(VgaChar {
                    ascii: byte,
                    color: color_code,
                });
                self.col_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for b in s.bytes() {
            match b {
                // We only keep printable ASCII and new lines
                0x20..=0x7e | b'\n' => self.write_byte(b),
                _ => self.write_byte(0xfe), // Print a ■ for unsupported characters
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.buffer.0[row][col].read();
                self.buffer.0[row - 1][col].write(ch);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.col_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.0[row][col].write(VgaChar{
                ascii: b' ',
                color: self.color_code,
            })
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static::lazy_static! {
    pub static ref WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer {
        col_pos: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($args:tt)*) => ($crate::print!("{}\n", format_args!( $($args)* )));
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::vga::_print( format_args! ( $($args)* )));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn print_byte() {
        print!("H");
    }

    #[test_case]
    fn print_string() {
        println!("Hello world!");
    }

    #[test_case]
    fn print_a_lot_of_string() {
        for i in 0..200 {
            println!("{}", i);
        }
    }
}
