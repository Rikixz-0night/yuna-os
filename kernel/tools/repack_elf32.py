#!/usr/bin/env python3
"""Repackage an ELF64 kernel as an ELF32 multiboot image.

QEMU's multiboot loader rejects ELFCLASS64 outright, but the kernel keeps a
genuine 32-bit entry stub (``_start``) that builds page tables and jumps into
long mode.  The *container* therefore has to advertise EM_386 / ELFCLASS32
while the payload keeps its 64-bit segments untouched.

Only the ELF header and program headers are rewritten; segment file offsets are
preserved because every PT_LOAD lives at/after offset 0x1000, past the header
area.  Bytes belonging to segments are copied verbatim.
"""

import struct
import sys

ELF64_EHDR = struct.Struct("<16sHHIQQQIHHHHHH")
ELF64_PHDR = struct.Struct("<IIQQQQQQ")
ELF32_EHDR = struct.Struct("<16sHHIIIIIHHHHHH")
ELF32_PHDR = struct.Struct("<IIIIIIII")

PT_LOAD = 1
EM_386 = 3
ELFCLASS32 = 1
ET_EXEC = 2
HEADER_ROOM = 0x1000


def repack(src: str, dst: str) -> None:
    data = bytearray(open(src, "rb").read())

    (ident, e_type, e_machine, e_version, e_entry, e_phoff, e_shoff,
     e_flags, e_ehsize, e_phentsize, e_phnum, e_shentsize, e_shnum,
     e_shstrndx) = ELF64_EHDR.unpack_from(data, 0)

    if ident[:4] != b"\x7fELF" or ident[4] != 2:
        raise SystemExit("input is not an ELF64 file")
    if e_phentsize != ELF64_PHDR.size:
        raise SystemExit("unexpected program header size")

    phdrs = []
    for i in range(e_phnum):
        phdrs.append(ELF64_PHDR.unpack_from(data, e_phoff + i * e_phentsize))

    loads = [p for p in phdrs if p[0] == PT_LOAD]
    if not loads:
        raise SystemExit("no PT_LOAD segments")

    # ELF64 Phdr: type, flags, offset, vaddr, paddr, filesz, memsz, align
    min_offset = min(p[2] for p in loads)
    if min_offset < ELF32_EHDR.size + ELF32_PHDR.size * len(phdrs):
        raise SystemExit("no room for 32-bit headers before first segment")

    out_ident = bytearray(ident)
    out_ident[4] = ELFCLASS32
    out_ident[5] = 1  # little endian
    out_ident[6] = 1  # EV_CURRENT

    header = ELF32_EHDR.pack(
        bytes(out_ident), ET_EXEC, EM_386, 1,
        e_entry, ELF32_EHDR.size, 0, 0,
        ELF32_EHDR.size, ELF32_PHDR.size, len(phdrs), 0, 0, 0,
    )

    body = bytearray()
    for (p_type, p_flags, p_offset, p_vaddr, p_paddr, p_filesz, p_memsz,
         p_align) in phdrs:
        for value, name in ((p_offset, "offset"), (p_vaddr, "vaddr"),
                            (p_paddr, "paddr"), (p_filesz, "filesz"),
                            (p_memsz, "memsz")):
            if value > 0xFFFF_FFFF:
                raise SystemExit(f"segment {name} does not fit in ELF32")
        body += ELF32_PHDR.pack(p_type, p_offset, p_vaddr, p_paddr,
                                p_filesz, p_memsz, p_flags, p_align)

    out = bytearray(header + body)
    if len(out) > min_offset:
        raise SystemExit("32-bit headers overlap the first segment")
    out += b"\x00" * (min_offset - len(out))
    out += data[min_offset:]

    open(dst, "wb").write(out)
    print(f"wrote {dst}: {len(out)} bytes, entry {e_entry:#x}, "
          f"{len(loads)} loadable segments")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(f"usage: {sys.argv[0]} <in.elf> <out.elf>")
    repack(sys.argv[1], sys.argv[2])
