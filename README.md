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
- **Root User**: `root` | **Password**: `Nika123h`
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
root password is `Nika123h`), prints the Yuna fastfetch banner and drops you
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

## Guided Installation (`yunainstall`)

Yuna OS includes **`yunainstall`**, a fast, zero-dependency Archinstall-style guided installer written in Rust. It prompts you step by step, provides curated recommendations for each subsystem, creates users with wheel `sudo` permissions, and performs automated disk formatting and system deployment.

```sh
sudo yunainstall
```

### Step-by-Step Installation Walkthrough

```text
    __  __ _   _ _  _    _
    \ \/ /| | | | \| |  /_\
     \  / | |_| | .` | / _ \
     /  \ |  _  | |\  |/ ___ \
    /_/\_\|_| |_|_| \_/_/   \_\
         Yuna OS installer
```

1. **Live Environment & Network Setup**:
   - Boot from the USB / ISO (UEFI or BIOS).
   - The live session autologins user `yuna` on `tty1` (password `yuna`, full `sudo` access).
   - Connect to Wi-Fi via `yuna-wifi` or `nmtui` (or plug an Ethernet cable).
   - Test or optimize Singapore mirrors: `sudo yuna-mirrors`.

2. **Launch the Installer**:
   - Run `sudo yunainstall` from the terminal.

3. **Select Target Disk**:
   - `yunainstall` lists all attached storage drives (`/dev/nvme0n1`, `/dev/sda`, etc.) with model and capacity.
   - Choose your target drive. *(Note: All existing partitions on this drive will be wiped and partitioned with a modern GPT layout)*.

4. **System Identity & User Creation**:
   - **Hostname**: Enter your machine name (default: `yuna`).
   - **Username**: Enter your primary non-root username (default: `yuna`).
   - **User Password**: Enter a secure password (input is masked for security).
   - **Root Password**: Choose whether to reuse the user password for `root` (Recommended: `Yes`) or enter a dedicated root password.
   - **Privilege Setup**: The created user is automatically enrolled in the `wheel` administrative group with full `sudo` rights enabled in `/etc/sudoers`.

5. **Locale & Timezone**:
   - **Timezone**: Default is preset to **`Asia/Phnom_Penh`** (Cambodia Khmer Time, UTC+7).
   - **Locale**: Default is `en_US.UTF-8` (full Khmer script and CJK anime fonts are pre-installed).
   - **Keymap**: Default is `us`.

6. **Storage & Filesystem (Recommendations)**:
   - **Filesystem Options**:
     - `btrfs` **(Recommended for Snapshots)**: Formats with automated `@` (root) and `@home` (user home) subvolumes, transparent `zstd` compression, and `noatime` mount flags. Ready for instant Timeshift/Snapper rollback snapshots.
     - `ext4` **(Recommended for Maximum Compatibility)**: Classic, rock-solid journaling filesystem with minimal overhead.
     - `f2fs` **(Recommended for NVMe/PCIe SSD Endurance)**: Flash-Friendly File System optimized for modern solid-state drives.
     - `xfs`: Enterprise-grade filesystem for massive throughput and large files.
   - **Disk Encryption**: Optional LUKS2 full-disk encryption with Argon2id PBKDF.
   - **ESP Size**: 512 MiB EFI System Partition (on UEFI systems).

7. **Linux Kernel Flavor (Recommendations)**:
   - `linux` **(Recommended)**: Latest upstream stable vanilla Linux kernel.
   - `linux-lts` **(Recommended for Rock-Solid Stability)**: Long-Term Support kernel for servers and critical workstations.
   - `linux-zen` **(Recommended for Desktop & Gaming)**: Tuned with low-latency kernel scheduler responsiveness.

8. **Desktop Environment (Recommendations)**:
   - `kde` **(Recommended for Full Modern Desktop)**: KDE Plasma 6 Wayland desktop with SDDM, Dolphin, and Konsole.
   - `hyprland` **(Recommended for Modern Tiling)**: Fluid dynamic Wayland compositor with Tuigreet, Waybar, Wofi, and Kitty.
   - `xfce` **(Recommended for Low-Spec / Minimal RAM)**: Ultra-lightweight classic desktop with LightDM and Thunar.
   - `gnome`: Modern streamlined GNOME Shell with GDM.
   - `sway`: i3-compatible Wayland tiling window manager.
   - `cinnamon`: Traditional sleek panel desktop with Nemo.
   - `none`: Minimalist console / server environment.

9. **Graphics Drivers (Recommendations)**:
   - Hardware detection auto-probes detected GPUs (Intel, AMD, NVIDIA).
   - `auto` **(Recommended for Intel/AMD/Open-Source)**: Installs `mesa` and open-source modesetting drivers.
   - `nvidia` **(Recommended for GeForce/RTX cards)**: Installs proprietary `nvidia-dkms`, modules, and sets `nvidia_drm.modeset=1`.
   - `intel` / `amd` / `none`.

10. **Memory & Swap (Recommendations)**:
    - `zram` **(Recommended)**: In-RAM compressed swap (`systemd-zram-generator`), provides zero SSD wear and ultra-fast memory paging.
    - `none`: Disables swap partition.

11. **Bootloader (Recommendations)**:
    - `systemd-boot` **(Recommended for UEFI)**: Native, blazing fast UEFI bootloader with custom boot entries.
    - `grub` **(Recommended for BIOS / Multi-boot)**: Universal bootloader supporting BIOS and complex disk topologies.

12. **Review Plan, Install & Reboot**:
    - `yunainstall` outputs a full summary table of all configuration values.
    - Confirm with `y` to write the changes to disk.
    - Once finished, remove the installation media and reboot:
      ```sh
      sudo reboot
      ```

### Unattended / Automated Installation

You can also run automated, non-interactive installations via configuration files:

```sh
# Generate template plan
yunainstall --example-config > my-plan.conf

# Preview the exact command pipeline without touching disk (Dry-Run)
yunainstall --dry-run --config my-plan.conf

# Perform automated installation
sudo yunainstall --config my-plan.conf --yes
```

Example configuration (`examples/yuna.conf`):

```ini
disk=/dev/nvme0n1
hostname=yuna
username=yuna
password=your-secure-password
timezone=Asia/Phnom_Penh
locale=en_US.UTF-8
keymap=us
filesystem=btrfs
kernel=linux
desktop=hyprland
drivers=auto
bootloader=systemd-boot
swap=zram
encrypt=false
esp_mib=512
```

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
