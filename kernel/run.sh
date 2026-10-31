#!/usr/bin/env bash
# Build Yuna OS and boot it in QEMU.
#
#   ./run.sh                              boot interactively (serial console)
#   printf 'help\nhalt\n' | ./run.sh      scripted session
#
# Requires: rustup target add x86_64-unknown-none, qemu-system-x86_64, python3.
set -euo pipefail

PROFILE="${PROFILE:-release}"
TARGET_DIR="target/x86_64-unknown-none/${PROFILE}"
OUT="target/yuna_os.multiboot.elf"

cargo build --profile "$PROFILE" --target x86_64-unknown-none
python3 tools/repack_elf32.py "${TARGET_DIR}/yuna_os" "$OUT"

exec qemu-system-x86_64 \
    -kernel "$OUT" \
    -serial stdio \
    -monitor none \
    -display none \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -no-reboot \
    -m 128M
