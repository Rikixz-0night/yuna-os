//! Yuna OS - a small bare-metal x86_64 kernel.
//!
//! Boot flow:
//!   1. QEMU loads this ELF via the Multiboot (v1) protocol at 1 MiB, in 32-bit
//!      protected mode, with interrupts disabled.
//!   2. `_start` (assembly, 32-bit) builds identity page tables, enables PAE +
//!      long mode, loads a 64-bit GDT and far-jumps into `long_mode` (64-bit).
//!   3. `long_mode` normalizes segment registers, sets the stack and calls
//!      `kernel_main` with the Multiboot magic + info pointer.

#![no_std]
#![no_main]

mod console;
mod io;
mod serial;
mod shell;
mod vga;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    r#"
.section .multiboot, "a"
.align 4
    .long 0x1BADB002          /* Multiboot v1 magic              */
    .long 0x00000003          /* page-align + memory info        */
    .long 0xE4524FFB          /* -(magic + flags)                */

.section .text._start, "ax"
.code32
.global _start
_start:
    cli
    lea esp, [stack_top]
    mov edi, eax              /* Multiboot magic  -> arg1        */
    mov esi, ebx              /* Multiboot info   -> arg2        */

    /* Zero PML4 + PDPT + PD (3 pages). */
    push edi
    push esi
    lea edi, [pml4]
    xor eax, eax
    mov ecx, 3 * 1024
    rep stosd
    pop esi
    pop edi

    /* PML4[0] -> PDPT (PAE/long-mode entries are 8 bytes wide) */
    lea eax, [pdpt]
    or eax, 0x003
    mov [pml4], eax
    mov dword ptr [pml4 + 4], 0

    /* PDPT[0] -> PD */
    lea eax, [pd]
    or eax, 0x003
    mov [pdpt], eax
    mov dword ptr [pdpt + 4], 0

    /* Identity-map the first 1 GiB with 2 MiB huge pages. */
    xor ecx, ecx
1:
    mov eax, ecx
    shl eax, 21
    or eax, 0x083              /* present | writable | huge      */
    mov [pd + ecx * 8], eax
    mov dword ptr [pd + ecx * 8 + 4], 0
    inc ecx
    cmp ecx, 512
    jne 1b

    /* Enable PAE. */
    mov eax, cr4
    or eax, 0x020
    mov cr4, eax

    lea eax, [pml4]
    mov cr3, eax

    /* Enable long mode (EFER.LME). */
    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100
    wrmsr

    /* Enable paging + protection. */
    mov eax, cr0
    or eax, 0x80000001
    mov cr0, eax

    lgdt [gdt64_ptr]

    /* Far jump into the 64-bit code segment. */
    push 0x08
    lea eax, [long_mode]
    push eax
    retf

.code64
long_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    lea rsp, [rip + stack_top]
    call kernel_main
2:  hlt
    jmp 2b

.section .rodata
.align 16
gdt64:
    .quad 0
    .quad 0x00AF9A000000FFFF   /* 64-bit kernel code             */
    .quad 0x00CF92000000FFFF   /* flat data                      */
.align 8
gdt64_ptr:
    .word 23                   /* 3 descriptors - 1              */
    .quad gdt64

.section .bss
.align 4096
pml4:
    .skip 4096
pdpt:
    .skip 4096
pd:
    .skip 4096
.align 16
stack_bottom:
    .skip 32768
stack_top:
"#
);

const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BAD_B002;

/// Multiboot v1 information structure (only the fields we use).
#[repr(C)]
struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
}

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, info: u32) -> ! {
    serial::init();
    vga::init();

    console::clear();

    println!();
    println!("  Y U N A   O S");
    println!("  ---------------");
    println!("  a small x86_64 kernel written in rust");
    println!();

    if magic == MULTIBOOT_BOOTLOADER_MAGIC && info != 0 {
        let mb = unsafe { &*(info as *const MultibootInfo) };
        if mb.flags & (1 << 0) != 0 {
            let total_kib = mb.mem_upper as u64 + 1024;
            shell::set_boot_memory(total_kib as u32);
            println!("[boot] multiboot ok, magic={:#010x}", magic);
            println!(
                "[boot] memory: lower={} KiB upper={} KiB total={} MiB",
                mb.mem_lower,
                mb.mem_upper,
                total_kib / 1024
            );
        } else {
            println!("[boot] multiboot ok, memory info unavailable");
        }
    } else {
        println!("[boot] warning: not booted by a multiboot loader");
    }

    println!("[boot] long mode active, paging on (1 GiB identity map)");
    println!("[boot] serial=COM1 115200 8N1   video=VGA text 80x25");
    println!();
    println!("type `help` to list commands.");
    println!();

    shell::run()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("*** kernel panic ***");
    if let Some(msg) = info.message().as_str() {
        println!("{}", msg);
    }
    if let Some(loc) = info.location() {
        println!("at {}:{}", loc.file(), loc.line());
    }
    halt()
}

/// Stops the CPU for good.
pub fn halt() -> ! {
    loop {
        unsafe { asm!("cli; hlt", options(nomem, nostack)) };
    }
}
