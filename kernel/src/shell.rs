//! A tiny line-oriented shell over the serial port.

use core::arch::asm;

use crate::{halt, io::outb, print, println, serial};

const MAX_LINE: usize = 128;

pub fn run() -> ! {
    let mut buffer = [0u8; MAX_LINE];
    let mut len = 0usize;

    prompt();

    loop {
        let byte = serial::read_byte();

        match byte {
            b'\r' | b'\n' => {
                // Echo the newline to both devices.
                serial::write_byte(b'\r');
                serial::write_byte(b'\n');
                crate::vga::write_byte(b'\n');

                let line = core::str::from_utf8(&buffer[..len]).unwrap_or("");
                execute(line.trim());

                buffer = [0u8; MAX_LINE];
                len = 0;
                prompt();
            }
            0x7F | 0x08 => {
                if len > 0 {
                    len -= 1;
                    serial::write_byte(0x08);
                    serial::write_byte(b' ');
                    serial::write_byte(0x08);
                    crate::vga::write_byte(0x08);
                }
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                if len < MAX_LINE {
                    buffer[len] = byte;
                    len += 1;
                    serial::write_byte(byte);
                    crate::vga::write_byte(byte);
                }
            }
            _ => {}
        }
    }
}

fn prompt() {
    print!("yuna> ");
}

fn execute(line: &str) {
    let (command, args) = match line.find(' ') {
        Some(index) => (&line[..index], line[index + 1..].trim()),
        None => (line, ""),
    };

    match command {
        "" => {}
        "help" => {
            println!("commands:");
            println!("  help         show this list");
            println!("  echo <text>  print text back");
            println!("  info         kernel / boot information");
            println!("  mem          report usable memory");
            println!("  ticks        cpu cycle counter (rdtsc)");
            println!("  clear        clear the screen");
            println!("  halt         power off the machine");
        }
        "echo" => println!("{}", args),
        "info" => {
            println!("kernel : yuna_os 0.1.0");
            println!("arch   : x86_64, long mode, 4-level paging");
            println!("target : x86_64-unknown-none (no std)");
            println!("video  : vga text 80x25 @ 0xb8000");
            println!("serial : com1 @ 0x3f8, 115200 8n1");
        }
        "mem" => {
            let total_kib = boot_memory_kib();
            match total_kib {
                Some(kib) => {
                    println!("usable memory: {} KiB ({} MiB)", kib, kib / 1024);
                }
                None => println!("memory information unavailable"),
            }
        }
        "ticks" => {
            let low: u32;
            let high: u32;
            unsafe {
                asm!("rdtsc", out("eax") low, out("edx") high, options(nomem, nostack));
            }
            let value = ((high as u64) << 32) | low as u64;
            println!("rdtsc: {}", value);
        }
        "clear" => crate::console::clear(),
        "halt" => {
            println!("shutting down...");
            // isa-debug-exit device, if present (used by run.sh).
            unsafe { outb(0xf4, 0x10) };
            halt();
        }
        other => println!("unknown command: {} (try `help`)", other),
    }
}

/// Reads mem_upper from the Multiboot info block that `_start` forwarded.
/// The kernel entry stores it for us via `boot_memory_kib`.
static mut TOTAL_KIB: Option<u32> = None;

pub fn set_boot_memory(total_kib: u32) {
    unsafe { TOTAL_KIB = Some(total_kib) };
}

fn boot_memory_kib() -> Option<u32> {
    unsafe { TOTAL_KIB }
}
