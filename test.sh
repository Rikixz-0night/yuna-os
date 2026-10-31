#!/usr/bin/env bash
#
# Yuna OS test suite.
#
#   ./test.sh                 everything (tools, installer, packages, profile, kernel)
#   ./test.sh --fast          skip package build and kernel boot
#   ./test.sh --no-kernel     skip the bare-metal boot test
#   ./test.sh --iso           also require and inspect a built ISO
#
# Nothing here needs root; the ISO build itself does (scripts/build-iso.sh).
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAST=0
NO_KERNEL=0
CHECK_ISO=0

for arg in "$@"; do
    case "$arg" in
        --fast) FAST=1 ;;
        --no-kernel) NO_KERNEL=1 ;;
        --iso) CHECK_ISO=1 ;;
        -h|--help)
            sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown option: $arg" >&2
            exit 2
            ;;
    esac
done

if [[ -t 1 ]]; then
    GREEN=$'\e[32m'; RED=$'\e[31m'; CYAN=$'\e[36m'; DIM=$'\e[2m'; YELLOW=$'\e[33m'; RESET=$'\e[0m'
else
    GREEN=""; RED=""; CYAN=""; DIM=""; YELLOW=""; RESET=""
fi

PASS=0
FAIL=0
SKIP=0
LOGDIR="$(mktemp -d)"
trap 'rm -rf "$LOGDIR"' EXIT

section() {
    echo
    echo "${CYAN}── $* ${RESET}"
}

ok() {
    PASS=$((PASS + 1))
    printf '  %sPASS%s %s\n' "$GREEN" "$RESET" "$*"
}

bad() {
    FAIL=$((FAIL + 1))
    printf '  %sFAIL%s %s\n' "$RED" "$RESET" "$*"
}

skip() {
    SKIP=$((SKIP + 1))
    printf '  %sSKIP%s %s\n' "$YELLOW" "$RESET" "$*"
}

# run <name> <command...>
run() {
    local name="$1"
    shift
    local logfile="$LOGDIR/check.log"
    if "$@" >"$logfile" 2>&1; then
        ok "$name"
    else
        bad "$name"
        echo "${DIM}    last lines:${RESET}"
        tail -n 8 "$logfile" | sed 's/^/    /'
    fi
}

# shell_run <name> <shell snippet>
shell_run() {
    local name="$1"
    shift
    run "$name" bash -c "$*"
}

# expect_failure <name> <expected substring> <command...>
expect_failure() {
    local name="$1"
    local expected="$2"
    shift 2
    local logfile="$LOGDIR/failure.log"

    if "$@" >"$logfile" 2>&1; then
        bad "$name (command unexpectedly succeeded)"
        return
    fi
    if grep -qi -- "$expected" "$logfile"; then
        ok "$name"
    else
        bad "$name (failed, but message did not contain '$expected')"
        tail -n 5 "$logfile" | sed 's/^/    /'
    fi
}

echo "${CYAN}"
cat <<'BANNER'
   __  __ _   _ _  _    _
   \ \/ /| | | | \| |  /_\
    \  / | |_| | .` | / _ \
    /  \ |  _  | |\  |/ ___ \
   /_/\_\|_| |_|_| \_/_/   \_\
BANNER
echo "        Yuna OS test suite${RESET}"

########################################
section "toolchain"
########################################
for tool in cargo rustc python3 bash makepkg repo-add; do
    if command -v "$tool" >/dev/null 2>&1; then
        ok "$tool present"
    else
        bad "$tool missing"
    fi
done

########################################
section "rust tooling"
########################################
run "cargo build --release" bash -c "cd '$ROOT/rust' && cargo build --release"
FETCH="$ROOT/rust/target/release/yuna-fetch"
INSTALLER="$ROOT/rust/target/release/yunainstall"
WIFI="$ROOT/rust/target/release/yuna-wifi"

if [[ -x "$FETCH" ]]; then
    run "yuna-fetch runs" "$FETCH" --no-logo
    shell_run "yuna-fetch --json is valid JSON" "'$FETCH' --json | python3 -m json.tool > /dev/null"
    shell_run "yuna-fetch --short produces output" "'$FETCH' --short | grep -q ."
else
    skip "yuna-fetch (not built)"
fi

########################################
section "yuna-wifi"
########################################
if [[ -x "$WIFI" ]]; then
    run "yuna-wifi --help" "$WIFI" --help
    run "yuna-wifi --version" "$WIFI" --version
    if command -v nmcli >/dev/null 2>&1; then
        run "yuna-wifi --status" "$WIFI" --status
    else
        skip "yuna-wifi live status (nmcli missing)"
    fi
else
    skip "yuna-wifi (not built)"
fi

########################################
section "yunainstall"
########################################
if [[ -x "$INSTALLER" ]]; then
    run "--help" "$INSTALLER" --help
    run "--list-disks" "$INSTALLER" --list-disks
    run "--example-config" "$INSTALLER" --example-config

    run "dry run with example plan" "$INSTALLER" --dry-run --yes --config "$ROOT/examples/yuna.conf"

    shell_run "dry run covers the whole pipeline" \
        "out=\$('$INSTALLER' --dry-run --yes --config '$ROOT/examples/yuna.conf'); \
         grep -q wipefs <<<\"\$out\" && grep -q sgdisk <<<\"\$out\" && \
         grep -q pacstrap <<<\"\$out\" && grep -q arch-chroot <<<\"\$out\""

    shell_run "generated chroot script is valid bash" \
        "'$INSTALLER' --config '$ROOT/examples/yuna.conf' --print-script | bash -n"

    shell_run "generated script contains the bootloader entry" \
        "'$INSTALLER' --config '$ROOT/examples/yuna.conf' --print-script | grep -q 'title   Yuna OS'"

    shell_run "generated script prepares the pacman keyring" \
        "'$INSTALLER' --config '$ROOT/examples/yuna.conf' --print-script | grep -q 'pacman-key --populate archlinux'"

    sed 's/^drivers=auto/drivers=nvidia/' "$ROOT/examples/yuna.conf" > "$LOGDIR/nvidia.conf"
    shell_run "drivers=nvidia pulls nvidia-dkms and the kernel argument" \
        "out=\$('$INSTALLER' --dry-run --yes --config '$LOGDIR/nvidia.conf'); \
         grep -q nvidia-dkms <<<\"\$out\" && grep -q nvidia_drm.modeset=1 <<<\"\$out\""

    sed 's/^drivers=auto/drivers=intel/' "$ROOT/examples/yuna.conf" > "$LOGDIR/intel.conf"
    shell_run "drivers=intel pulls vulkan-intel" \
        "'$INSTALLER' --dry-run --yes --config '$LOGDIR/intel.conf' | grep -q vulkan-intel"

    shell_run "dry run does not create /mnt/root/yuna-post.sh" \
        "test ! -e /mnt/root/yuna-post.sh"

    cat > "$LOGDIR/bad.conf" <<'EOF'
disk=/dev/sda
hostname=ok
username=Bad Name
password=x
timezone=UTC
locale=en_US.UTF-8
keymap=us
filesystem=ext4
kernel=linux
desktop=none
bootloader=grub
swap=none
encrypt=false
esp_mib=512
EOF
    expect_failure "invalid username is rejected" "invalid username" \
        "$INSTALLER" --dry-run --yes --config "$LOGDIR/bad.conf"

    cat > "$LOGDIR/bad-key.conf" <<'EOF'
disk=/dev/sda
password=x
nonsense=1
EOF
    expect_failure "unknown config key is rejected" "unknown key" \
        "$INSTALLER" --dry-run --yes --config "$LOGDIR/bad-key.conf"
else
    skip "yunainstall (not built)"
fi

########################################
section "packaging"
########################################
if (( FAST )); then
    skip "package build (--fast)"
else
    run "build-repo.sh" "$ROOT/scripts/build-repo.sh"
    for package in yuna-tools yuna-release; do
        shell_run "$package package exists" "compgen -G '$ROOT/out/repo/$package-*.pkg.tar.zst' > /dev/null"
    done
    shell_run "repository database is valid" \
        "bsdtar -tf '$ROOT/out/repo/yuna.db.tar.gz' | grep -q 'yuna-tools'"
    shell_run "yuna-tools ships yuna-wifi" \
        "bsdtar -tf '$ROOT'/out/repo/yuna-tools-*.pkg.tar.zst | grep -q 'usr/bin/yuna-wifi'"
    shell_run "packages do not overwrite core-owned files" \
        "! bsdtar -tf '$ROOT'/out/repo/yuna-release-*.pkg.tar.zst | grep -qE '^(etc|usr/lib)/os-release$|^etc/issue$' && \
         ! bsdtar -tf '$ROOT'/out/repo/yuna-tools-*.pkg.tar.zst | grep -qE '^etc/skel/\.bashrc$|^etc/xdg/reflector/reflector.conf$'"
fi

########################################
section "archiso profile"
########################################
run "check-profile.sh" "$ROOT/scripts/check-profile.sh"
for script in "$ROOT"/scripts/*.sh "$ROOT/distro/profiledef.sh" \
              "$ROOT/distro/airootfs/usr/local/bin/yuna-welcome" \
              "$ROOT/distro/airootfs/usr/local/bin/yuna-mirrors" \
              "$ROOT/distro/airootfs/root/customize_airootfs.sh" \
              "$ROOT/distro/airootfs/etc/profile.d/yuna.sh"; do
    if bash -n "$script" 2>"$LOGDIR/syntax.log"; then
        ok "bash syntax: ${script#$ROOT/}"
    else
        bad "bash syntax: ${script#$ROOT/}"
        cat "$LOGDIR/syntax.log" | sed 's/^/    /'
    fi
done

########################################
section "bare-metal kernel"
########################################
if (( NO_KERNEL )) || (( FAST )); then
    skip "kernel boot (disabled)"
elif ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
    skip "kernel boot (qemu-system-x86_64 missing)"
else
    if run "kernel builds for x86_64-unknown-none" \
        bash -c "cd '$ROOT/kernel' && cargo build --release --target x86_64-unknown-none"; then
        run "multiboot image is generated" \
            bash -c "cd '$ROOT/kernel' && python3 tools/repack_elf32.py target/x86_64-unknown-none/release/yuna_os target/yuna_os.multiboot.elf"

        boot_log="$LOGDIR/kernel-boot.log"
        boot_status=0
        printf 'mem\ninfo\nhalt\n' | timeout 30 qemu-system-x86_64 \
                -kernel "$ROOT/kernel/target/yuna_os.multiboot.elf" \
                -serial stdio -monitor none -display none \
                -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
                -no-reboot -m 128M >"$boot_log" 2>&1 || boot_status=$?

        # `halt` writes 0x10 to the isa-debug-exit port, so QEMU terminates with
        # status (0x10 << 1) | 1 == 33. That is the expected clean exit, not a
        # failure; a timeout (124) or a crash is.
        if (( boot_status == 33 )); then
            ok "kernel halted through the debug-exit device (status 33)"
        elif (( boot_status == 0 )); then
            ok "kernel exited cleanly (status 0)"
        else
            bad "kernel exited with status $boot_status (expected 33 from halt)"
            tail -n 8 "$boot_log" | sed 's/^/    /'
        fi

        if grep -q "long mode active" "$boot_log" && grep -q "usable memory" "$boot_log"; then
            ok "kernel boots into long mode and answers the shell"
        else
            bad "kernel booted but output was incomplete"
            tail -n 8 "$boot_log" | sed 's/^/    /'
        fi
    fi
fi

########################################
section "ISO"
########################################
iso="$(compgen -G "$ROOT/out/*.iso" | head -n 1 || true)"
if [[ -n "$iso" ]]; then
    ok "ISO present: ${iso#$ROOT/} ($(du -h "$iso" | cut -f1))"
    if command -v xorriso >/dev/null 2>&1; then
        run "ISO volume is readable" xorriso -indev "$iso" -toc
    fi
elif (( CHECK_ISO )); then
    bad "no ISO found in out/ (run: sudo ./scripts/build-iso.sh)"
else
    skip "no ISO built yet (sudo ./scripts/build-iso.sh)"
fi

########################################
echo
printf '%s────────────────────────────────────────%s\n' "$DIM" "$RESET"
printf '  %s%d passed%s   %s%d failed%s   %s%d skipped%s\n' \
    "$GREEN" "$PASS" "$RESET" "$RED" "$FAIL" "$RESET" "$YELLOW" "$SKIP" "$RESET"
printf '%s────────────────────────────────────────%s\n' "$DIM" "$RESET"

if (( FAIL > 0 )); then
    exit 1
fi
exit 0
