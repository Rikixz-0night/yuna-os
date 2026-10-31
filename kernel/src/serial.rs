//! COM1 serial driver (16550 UART, polled, no interrupts).

use crate::io::{inb, outb};

const COM1: u16 = 0x3F8;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00); // disable interrupts
        outb(COM1 + 3, 0x80); // enable DLAB
        outb(COM1 + 0, 0x01); // divisor low  (115200 baud)
        outb(COM1 + 1, 0x00); // divisor high
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop
        // NOTE: writing FCR flushes a byte that arrived before boot, so the
        // FIFO is left at its reset state (1-byte RX holding register).
        outb(COM1 + 4, 0x0B); // IRQs off, RTS/DSR set
    }
}

fn line_status() -> u8 {
    unsafe { inb(COM1 + 5) }
}

pub fn write_byte(byte: u8) {
    while line_status() & 0x20 == 0 {}
    unsafe { outb(COM1, byte) }
}

pub fn try_read_byte() -> Option<u8> {
    if line_status() & 0x01 != 0 {
        Some(unsafe { inb(COM1) })
    } else {
        None
    }
}

/// Blocks until at least one byte arrives.
pub fn read_byte() -> u8 {
    loop {
        if let Some(byte) = try_read_byte() {
            return byte;
        }
        core::hint::spin_loop();
    }
}
