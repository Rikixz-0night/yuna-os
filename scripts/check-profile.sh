#!/usr/bin/env bash
# Static validation of the archiso profile. No root required.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DISTRO="$ROOT/distro"
failures=0

fail() { echo "  FAIL $*"; failures=$((failures + 1)); }
ok()   { echo "  ok   $*"; }

echo "== required files =="
required=(
    profiledef.sh
    packages.x86_64
    pacman.conf
    airootfs/etc/os-release
    airootfs/etc/issue
    airootfs/etc/motd
    airootfs/etc/pacman.conf
    airootfs/etc/pacman.d/mirrorlist
    airootfs/etc/xdg/reflector/reflector.conf
    airootfs/etc/mkinitcpio.conf.d/archiso.conf
    airootfs/etc/skel/.config/fastfetch/config.jsonc
    airootfs/usr/share/yuna/logo.txt
    airootfs/usr/local/bin/yuna-welcome
    airootfs/usr/local/bin/yuna-mirrors
    airootfs/root/customize_airootfs.sh
    efiboot/loader/loader.conf
    efiboot/loader/entries/yuna.conf
    syslinux/syslinux.cfg
    syslinux/archiso_head.cfg
    syslinux/archiso_sys.cfg
    syslinux/archiso_sys-linux.cfg
    syslinux/archiso_tail.cfg
)
for path in "${required[@]}"; do
    if [[ -e "$DISTRO/$path" ]]; then
        ok "$path"
    else
        fail "missing $path"
    fi
done

echo "== file_permissions targets =="
while IFS= read -r path; do
    [[ -z "$path" ]] && continue
    case "$path" in
        /etc/shadow|/etc/gshadow)
            ok "airootfs$path (provided by base packages)"
            continue
            ;;
    esac
    if [[ -e "$DISTRO/airootfs$path" ]]; then
        ok "airootfs$path"
    else
        fail "file_permissions references missing airootfs$path"
    fi
done < <(grep -oP '^\s*\["\K[^"]+' "$DISTRO/profiledef.sh")

echo "== boot placeholders =="
for marker in %INSTALL_DIR% %ARCHISO_UUID% %ARCH%; do
    if grep -Rq -- "$marker" "$DISTRO/syslinux" "$DISTRO/efiboot"; then
        ok "$marker used"
    else
        fail "$marker missing from boot configs"
    fi
done

echo "== package list =="
duplicates="$(sort "$DISTRO/packages.x86_64" | uniq -d)"
if [[ -z "$duplicates" ]]; then
    ok "no duplicate packages"
else
    fail "duplicate packages: $duplicates"
fi

for package in yuna-tools yuna-release; do
    grep -qx "$package" "$DISTRO/packages.x86_64" \
        && ok "$package included" \
        || fail "$package missing from packages.x86_64"
done

echo "== official packages exist =="
if pacman -Si base >/dev/null 2>&1; then
    missing=0
    checked=0
    while IFS= read -r package; do
        [[ -z "$package" || "$package" == \#* ]] && continue
        [[ "$package" == yuna-* ]] && continue
        checked=$((checked + 1))
        if ! pacman -Si "$package" >/dev/null 2>&1; then
            fail "package not in enabled repositories: $package"
            missing=$((missing + 1))
        fi
    done < "$DISTRO/packages.x86_64"
    (( missing == 0 )) && ok "$checked packages found in repositories"
else
    echo "  skip (no pacman sync database)"
fi

echo "== network and mirrors =="
for package in networkmanager wpa_supplicant iw iwd wireless-regdb linux-firmware; do
    grep -qx "$package" "$DISTRO/packages.x86_64" \
        && ok "wifi package: $package" \
        || fail "$package missing from packages.x86_64"
done

mirror_count="$(grep -c '^Server = https://' "$DISTRO/airootfs/etc/pacman.d/mirrorlist" || true)"
if (( mirror_count >= 3 )); then
    ok "$mirror_count HTTPS Arch mirrors"
else
    fail "only $mirror_count HTTPS mirror(s) in the live mirrorlist"
fi

grep -q 'Include = /etc/pacman.d/mirrorlist' "$DISTRO/airootfs/etc/pacman.conf" \
    && ok "runtime pacman.conf uses the mirrorlist" \
    || fail "runtime pacman.conf does not include the mirrorlist"

grep -q 'systemctl enable NetworkManager' "$DISTRO/airootfs/root/customize_airootfs.sh" \
    && ok "NetworkManager enabled at build time" \
    || fail "customize_airootfs.sh does not enable NetworkManager"

grep -q 'pacman-key --populate archlinux' "$DISTRO/airootfs/root/customize_airootfs.sh" \
    && ok "pacman keyring populated at build time" \
    || fail "customize_airootfs.sh does not populate the pacman keyring"

echo "== live initramfs =="
grep -qx 'mkinitcpio-archiso' "$DISTRO/packages.x86_64" \
    && ok "mkinitcpio-archiso included" \
    || fail "mkinitcpio-archiso missing from packages.x86_64"

grep -q 'archiso' "$DISTRO/airootfs/etc/mkinitcpio.conf.d/archiso.conf" \
    && ok "archiso hook configured" \
    || fail "archiso hook missing from mkinitcpio config"

grep -q 'mkinitcpio -P' "$DISTRO/airootfs/root/customize_airootfs.sh" \
    && ok "live initramfs is rebuilt at build time" \
    || fail "customize_airootfs.sh does not run mkinitcpio -P"

echo "== pacman repo stanzas =="
for section in core extra yuna; do
    grep -q "^\[$section\]" "$DISTRO/pacman.conf" \
        && ok "build pacman.conf has [$section]" \
        || fail "build pacman.conf missing [$section]"
    grep -q "^\[$section\]" "$DISTRO/airootfs/etc/pacman.conf" \
        && ok "runtime pacman.conf has [$section]" \
        || fail "runtime pacman.conf missing [$section]"
done

echo
if (( failures > 0 )); then
    echo "$failures check(s) failed"
    exit 1
fi
echo "profile OK"
