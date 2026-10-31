# Yuna OS (Cambodia Anime Edition)

> **In Active Development**  
> **Author & Creator**: **Rikixz**  
> **Dedication**: Made for Cambodia community and anime lovers (*For Cambodia com anime*)  
> **GitHub**: [https://github.com/Rikixz-0night/yuna-os](https://github.com/Rikixz-0night/yuna-os)  
> **Architecture & Roadmap**: [DEVELOPMENT.md](DEVELOPMENT.md)

An Arch-based Linux distribution with Rust-native tooling, out-of-the-box font
support (Khmer + Asian CJK anime typography), TLS and UFW firewall security,
preset Cambodia timezone, Singapore high-speed mirrors, plus the small bare-metal
x86_64 kernel from the first iteration (kept in `kernel/`).

Pacman underneath. Rust where it matters. `fastfetch` wired in, and an
archinstall-style installer called **`yunainstall`**.

```
      ██╗   ██╗██╗   ██╗███╗   ██╗ █████╗
      ╚██╗ ██╔╝██║   ██║████╗  ██║██╔══██╗
       ╚████╔╝ ██║   ██║██╔██╗ ██║███████║
        ╚██╔╝  ██║   ██║██║╚██╗██║██╔══██║
         ██║   ╚██████╔╝██║ ╚████║██║  ██║
         ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝
```

## Quick Start & Live Medium

- **Live User**: `yuna` | **Password**: `yuna` (full `sudo` permissions)
- **Root User**: `root` | **Password**: `yuna`
- **Default Timezone**: `Asia/Phnom_Penh` (Cambodia Khmer Time)
- **Default Mirrors**: Singapore HTTPS mirrors for Southeast Asia low latency
- **Firewall**: `ufw` pre-installed (`firewall` alias, `sudo ufw status`)
- **Fonts**: Full Khmer (`noto-fonts`), Japanese/CJK anime (`noto-fonts-cjk`), emoji (`noto-fonts-emoji`), DejaVu, Liberation, and Terminus console fonts
- **Boot Menus**: Custom anime-themed Syslinux (BIOS) & systemd-boot (UEFI) menus with Default, Safe Graphics (`nomodeset`), and Copy to RAM modes

## What is in the box

| path | what |
|------|------|
| `distro/` | archiso profile: live environment, boot menus, branding, offline `[yuna]` repo |
| `rust/yuna-fetch/` | native system info tool (no dependencies, reads `/proc` + `/sys`) |
| `rust/yuna-wifi/` | console Wi-Fi client: scan, pick a network, connect |
| `rust/yunainstall/` | guided installer, archinstall-style, no dependencies |
| `packages/` | `yuna-tools` and `yuna-release` PKGBUILDs |
| `scripts/` | local repository and ISO builds |
| `docs/banner.md` | the portrait ASCII banner |
| `kernel/` | the bare-metal x86_64 kernel from the first iteration |

## Build the ISO (Arch host)

```sh
sudo pacman -S --needed archiso

./scripts/build-repo.sh        # builds yuna-* packages + local pacman repo
sudo ./scripts/build-iso.sh    # mkarchiso -> out/yuna-<date>-x86_64.iso
```

Boot it:

```sh
qemu-system-x86_64 -enable-kvm -m 4096 -cdrom out/yuna-*.iso
```

The live system autologins `yuna` on tty1 (password `yuna`, full sudo access;
root password is also `yuna`), prints the Yuna fastfetch banner and drops you
at a shell. NetworkManager is enabled, the UFW firewall is available, and the
pacman keyring is already populated, so `pacman -S anything` works immediately.

## Wi-Fi on the live medium

The ISO ships the normal Arch wireless stack:

- `linux-firmware` (Intel, AMD, Realtek, MediaTek/Ralink, Atheros, Broadcom)
  plus `wireless-regdb`
- NetworkManager + `wpa_supplicant`, and `iwd`/`iwctl` as an alternative
- `yuna-wifi`, a small Rust console client so nobody has to remember `nmcli`

```
yuna-wifi              interactive: scan, choose, type the password
yuna-wifi --list       list visible networks
yuna-wifi --status     show the current connection
yuna-wifi --connect "My Wi-Fi" --password secret
```

`nmtui` is the full NetworkManager text UI, and the raw commands are:

```sh
nmcli device wifi list
nmcli device wifi connect "SSID" password "KEY"
```

Firmware is loaded by the kernel automatically; `dmesg | grep -i firmware`
shows whether a card needs an extra blob. `lsusb`/`lspci` identify the chipset.
Connections made here are immediately available to the installer and to
`pacman`. The installer installs the same firmware, NetworkManager and
`iwd`/`iwctl` (disabled by default, as a rescue tool) on the target system.

## Mirrors and pacman

The live medium ships a fast Arch mirrorlist configured with Singapore HTTPS
mirrors at `/etc/pacman.d/mirrorlist` (optimized for Cambodia and Southeast Asia),
the pacman keyring is initialized during the ISO build, and `[core]`, `[extra]`
plus the offline `[yuna]` repository are enabled. That means the live session
and the installer install packages straight from Arch at low latency.

```sh
sudo yuna-mirrors      # reflector: rank Singapore / SE Asia mirrors
sudo pacman -Syu
sudo pacman -S firefox
```

The installed system keeps the Arch mirrorlist, gets `reflector` refreshing it
during the install step (tuned for Singapore / Southeast Asia), and
`pacman-key --init/--populate` is re-run in the chroot so signatures keep working.

Branding is packaged carefully: `/etc/os-release` and `/etc/issue` belong to the
Arch `filesystem` package, so `yuna-release` ships them under `/usr/lib/yuna`
and a scriptlet copies them into place. A pacman hook re-applies them after a
`filesystem` upgrade, so `pacman -Syu` never turns Yuna OS back into Arch.

## Install it

```sh
yunainstall
```

Guided flow: disk selection, hostname/user/passwords, timezone/locale/keymap,
filesystem, kernel, desktop, graphics drivers, swap, LUKS2, bootloader, then a
full plan summary before anything is written.

Non-interactive:

```sh
yunainstall --dry-run --config examples/yuna.conf   # preview every command
sudo yunainstall --config examples/yuna.conf --yes  # do it
```

| option | values |
|--------|--------|
| filesystem | `ext4`, `btrfs` (with `@` / `@home` subvolumes & zstd), `xfs`, `f2fs` (flash/NVMe optimized) |
| kernel | `linux`, `linux-lts`, `linux-zen` |
| desktop | `none`, `kde`, `xfce` (or `xfce3`/`xfce4`), `hyprland`, `gnome`, `sway`, `cinnamon` |
| drivers | `auto` (open-source), `none`, `intel`, `amd`, `nvidia` (proprietary `nvidia-dkms`) |
| bootloader | `systemd-boot` (UEFI), `grub` (UEFI + BIOS) |
| swap | `none`, `zram` |
| encryption | LUKS2 on the root partition |
| firmware | UEFI GPT layout or BIOS (bios_grub) layout, detected automatically |

The installer validates every string it writes (no shell injection from
hostname/user/timezone), base64-wraps passwords before they touch the chroot
script, refuses mounted disks, and requires root only for the real run.

## Live medium details

- `fastfetch` is installed and configured in `/etc/skel/.config/fastfetch/`
  with the Yuna logo at `/usr/share/yuna/logo.txt` and the `wifi` module on
- `yuna-fetch` is the Rust-native equivalent, available as `yuna-fetch`
- the offline `[yuna]` repository lives at `/opt/yuna/repo` on the ISO
- `yuna-welcome` prints the machine, the Wi-Fi state, the disks and the next steps
- `yuna-wifi` connects to wireless networks from the console
- BIOS (syslinux) and UEFI (systemd-boot) boot entries, plus a `nomodeset` entry
- `yuna-mirrors` ranks and rewrites the Arch mirrorlist

## Package repository

```sh
./scripts/build-repo.sh
# out/repo/yuna.db.tar.gz
# out/repo/yuna-tools-*.pkg.tar.zst
# out/repo/yuna-release-*.pkg.tar.zst
```

`yuna-tools` ships `yuna-fetch`, `yunainstall`, `yuna-wifi`, `yuna-mirrors`,
the fastfetch config and the shell aliases. `yuna-release` ships the branding
under `/usr/lib/yuna` and applies `/etc/os-release`, `/etc/issue` and
`/etc/motd` from an install scriptlet plus a pacman hook (see below).

## Verified locally

The whole suite runs without root:

```sh
./test.sh          # or: make test
./test.sh --fast   # skip the package build and the QEMU kernel boot
```

It covers the Rust build, `yuna-fetch`, `yuna-wifi`, every `yunainstall` mode
(including rejecting bad configs, driver selection and syntax-checking the
generated chroot script), the package build and repository database, the
archiso profile, and a real QEMU boot of the bare-metal kernel.

```sh
$ cargo build --release            # yuna-fetch + yunainstall, no warnings
$ ./scripts/build-repo.sh          # both packages + repo built
$ yunainstall --dry-run --yes --config examples/yuna.conf
```

The dry run prints the whole pipeline: `wipefs`, `sgdisk`, `cryptsetup`,
`mkfs`, `pacstrap`, the generated `/root/yuna-post.sh` (users, locale,
bootloader entries), and `arch-chroot` — without touching a disk.

## Banner

The portrait ASCII banner lives in `docs/banner.md`:

```sh
sed -n '/^```text$/,/^```$/p' docs/banner.md | sed '1d;$d'
```

## Kernel experiment

`kernel/` still builds and boots its own x86_64 kernel in QEMU:

```sh
cd kernel && ./run.sh
```

See `kernel/README.md`.
