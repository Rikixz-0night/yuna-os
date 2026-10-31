//! Console: every character is mirrored to both VGA and serial.

use core::fmt::{self, Write};

use crate::{serial, vga};

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    vga::write_byte(b'\n');
                    serial::write_byte(b'\r');
                    serial::write_byte(b'\n');
                }
                byte => {
                    vga::write_byte(byte);
                    serial::write_byte(byte);
                }
            }
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let _ = Console.write_fmt(args);
}

pub fn clear() {
    vga::clear();
    // ANSI clear-screen + home for the serial side.
    serial::write_byte(b'\x1b');
    serial::write_byte(b'[');
    serial::write_byte(b'2');
    serial::write_byte(b'J');
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}
