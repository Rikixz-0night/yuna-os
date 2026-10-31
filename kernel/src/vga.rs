//! VGA text-mode driver: 80x25, 16 colors, at physical 0xB8000.

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

const COLOR: u8 = 0x0F; // light grey on black

static mut ROW: usize = 0;
static mut COL: usize = 0;

pub fn init() {
    unsafe {
        ROW = 0;
        COL = 0;
    }
    clear();
}

pub fn clear() {
    for y in 0..VGA_HEIGHT {
        for x in 0..VGA_WIDTH {
            unsafe { put_raw(x, y, b' ', COLOR) };
        }
    }
    unsafe {
        ROW = 0;
        COL = 0;
    }
}

unsafe fn put_raw(x: usize, y: usize, byte: u8, color: u8) {
    let offset = (y * VGA_WIDTH + x) * 2;
    *VGA_BUFFER.add(offset) = byte;
    *VGA_BUFFER.add(offset + 1) = color;
}

fn scroll() {
    // Move every row up by one.
    for y in 1..VGA_HEIGHT {
        for x in 0..VGA_WIDTH {
            let src = (y * VGA_WIDTH + x) * 2;
            let dst = ((y - 1) * VGA_WIDTH + x) * 2;
            unsafe {
                let byte = *VGA_BUFFER.add(src);
                let color = *VGA_BUFFER.add(src + 1);
                *VGA_BUFFER.add(dst) = byte;
                *VGA_BUFFER.add(dst + 1) = color;
            }
        }
    }
    // Clear the last row.
    for x in 0..VGA_WIDTH {
        unsafe { put_raw(x, VGA_HEIGHT - 1, b' ', COLOR) };
    }
    unsafe { ROW = VGA_HEIGHT - 1 };
}

fn newline() {
    unsafe {
        COL = 0;
        ROW += 1;
        if ROW >= VGA_HEIGHT {
            scroll();
        }
    }
}

fn put(byte: u8) {
    match byte {
        b'\n' => newline(),
        b'\r' => unsafe { COL = 0 },
        0x08 => unsafe {
            // backspace
            if COL > 0 {
                COL -= 1;
                put_raw(COL, ROW, b' ', COLOR);
            }
        },
        byte => unsafe {
            if COL >= VGA_WIDTH {
                newline();
            }
            put_raw(COL, ROW, byte, COLOR);
            COL += 1;
            if COL >= VGA_WIDTH {
                newline();
            }
        },
    }
}

pub fn write_byte(byte: u8) {
    put(byte);
}
