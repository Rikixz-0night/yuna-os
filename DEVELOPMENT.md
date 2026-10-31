# Yuna OS — Development & Architecture Guide

> **Status**: Active Development  
> **Author & Lead Developer**: **Rikixz**  
> **Dedication**: Made for the **Cambodia Community & Anime Fans** (*For Cambodia com anime*)  
> **Repository**: [https://github.com/Rikixz-0night/yuna-os](https://github.com/Rikixz-0night/yuna-os)

```text
       ██╗   ██╗██╗   ██╗███╗   ██╗ █████╗
       ╚██╗ ██╔╝██║   ██║████╗  ██║██╔══██╗
        ╚████╔╝ ██║   ██║██╔██╗ ██║███████║
         ╚██╔╝  ██║   ██║██║╚██╗██║██╔══██║
          ██║   ╚██████╔╝██║ ╚████║██║  ██║
          ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝
  Cambodia Anime Edition • Arch Base • Rust Soul
```

---

## 1. Overview & Vision

**Yuna OS** is a modern, performance-oriented Linux distribution built on an Arch Linux base with Rust-native system utilities, created by **Rikixz** specifically for Cambodia and the anime enthusiast community.

Yuna OS combines:
- The rolling-release flexibility of **Arch Linux** and the official pacman repositories.
- Custom **Rust-native tooling** (`yuna-fetch`, `yuna-wifi`, and `yunainstall`) with zero external runtime dependencies.
- Beautiful out-of-the-box typography tailored for **Khmer script**, **Japanese / CJK anime fonts**, and console readability.
- Out-of-the-box network security with **TLS / SSL** certificates and **UFW firewall**.
- Localization preset for **Cambodia Khmer Timezone (`Asia/Phnom_Penh`)** and **Singapore HTTPS mirrors** for blazing fast downloads in Southeast Asia.
- A small experimental **bare-metal x86_64 kernel** in `kernel/` written in pure Rust (`#![no_std]`).

---

## 2. Core Features & Enhancements

### 🇰🇭 Cambodia Timezone & Singapore Fast Mirrors
- **Default Timezone**: Preset to `Asia/Phnom_Penh` across the live ISO environment, systemd clock configuration, and the `yunainstall` guided installer.
- **Singapore Mirrors**: Default pacman mirrorlist starts with high-speed Singapore HTTPS mirrors (`singapore.mirror.pkgbuild.com`, `sg.mirrors.cicku.me`, `mirror.aktkn.sg`), providing the lowest possible latency and highest throughput for Cambodian internet service providers.
- **Reflector Integration**: `yuna-mirrors` and `reflector.conf` are pre-tuned with `--country Singapore`.

### 🌸 Comprehensive Font Stack
Full unicode, Asian CJK, and Khmer font support pre-installed in the live image and target profiles:
- `noto-fonts` (Google Noto Sans & Serif, including complete Khmer script glyphs)
- `noto-fonts-cjk` (Full Chinese, Japanese, and Korean glyph support for manga, light novels, and anime media)
- `noto-fonts-emoji` (Color emoji support)
- `ttf-dejavu` & `ttf-liberation` (Versatile proportional and monospace fonts)
- `terminus-font` (Crisp bitmap font for terminal virtual consoles)

### 👤 Live Credentials
- **Live User**: `yuna` | **Password**: `yuna` (configured with passwordless `sudo` and full group permissions: `wheel`, `audio`, `video`, `storage`, `network`, `power`)
- **Root User**: `root` | **Password**: `yuna`
- **Autologin**: Automatic login to `yuna` on tty1 with `fastfetch` system banner and `yuna-welcome` diagnostics.

### 🛡️ Security, TLS & Firewall
- **TLS / SSL**: `ca-certificates`, `ca-certificates-utils`, `openssl`, `gnutls`, `curl`, and `wget` installed and verified. `systemd-timesyncd` synchronizes network time on boot to prevent certificate validation errors.
- **Firewall**: `ufw` (Uncomplicated Firewall) and `iptables` / `nftables` enabled with systemd service presets. Aliased as `firewall` for quick status and rule inspections.

### 🐧 Linux Kernel & Systemd
- **Kernel Flavors**: Selectable in `yunainstall` between `linux` (stable upstream), `linux-lts` (long-term support), and `linux-zen` (low-latency desktop tuning), with automatic DKMS module rebuilds for NVIDIA proprietary graphics (`nvidia-dkms`).
- **Systemd Suite**: Clean systemd service topology:
  - `NetworkManager` for wired and wireless connectivity
  - `systemd-timesyncd` for NTP clock sync
  - `systemd-resolved` for secure DNS resolution
  - `ufw` for perimeter network security
  - `fstrim.timer` for SSD maintenance
  - `zram-generator` for RAM-compressed swap

### 💾 Advanced Disk & Filesystem Stack
- **Filesystem Support**:
  - `ext4`: Reliable, widely supported standard Linux filesystem.
  - `btrfs`: Modern Copy-on-Write filesystem formatted with automatic `@` (root) and `@home` (home directory) subvolumes, transparent `zstd` compression, and `noatime` mount flags for optimal NVMe/SSD longevity and instant snapshotting (Timeshift/Snapper ready).
  - `xfs`: High-performance filesystem designed for large files and heavy I/O throughput.
  - `f2fs`: Flash-Friendly File System designed from the ground up for modern NVMe drives, PCIe SSDs, eMMC, and flash cards.
- **Disk Management Tools**: Pre-installed in the live environment and installed base:
  - `smartmontools` (`smartctl` S.M.A.R.T. disk health diagnostics)
  - `nvme-cli` (NVMe controller and namespace management)
  - `ntfs-3g` (full read/write support for Windows NTFS disks)
  - `exfatprogs` (exFAT USB drives and SD cards)
  - `f2fs-tools` (`mkfs.f2fs`, `fsck.f2fs`)
  - `gptfdisk`, `parted`, `dosfstools`, `mtools`, `e2fsprogs`, `btrfs-progs`, `xfsprogs`
  - `cryptsetup` (LUKS2 full-disk encryption with Argon2id key derivation)

### 🖥️ Desktop Environments
Selectable interactively during `yunainstall` or via config files:
- **KDE Plasma** (`desktop=kde`): Modern, beautiful desktop with SDDM display manager, Dolphin file manager, and Konsole.
- **Xfce** (`desktop=xfce` / `xfce4` / `xfce3`): Classic, lightweight, ultra-responsive desktop with LightDM, Thunar, and Whisker menu.
- **Hyprland** (`desktop=hyprland`): Bleeding-edge Wayland tiling compositor with smooth animations, Greetd/Tuigreet login, Waybar, Wofi, and Kitty.
- **GNOME** (`desktop=gnome`): Polished, distraction-free environment powered by GDM and Wayland.
- **Sway** (`desktop=sway`): High-speed, i3-compatible Wayland tiling window manager with Greetd/Tuigreet, Waybar, and Foot terminal.
- **Cinnamon** (`desktop=cinnamon`): Elegant, traditional desktop layout with LightDM and Nemo file manager.
- **Console** (`desktop=none`): Minimalist server/rescue installation.

### 🎨 Custom Boot Menu
- **Syslinux (BIOS)**:
  - Custom anime-styled magenta and cyan highlight color palette (`archiso_head.cfg`)
  - `Yuna OS Live (Default - Singapore Mirror, Cambodia Timezone)`
  - `Yuna OS Live (Copy to RAM)`
  - `Yuna OS Live (Safe Graphics nomodeset)`
  - Existing OS boot, reboot, and power-off options
- **systemd-boot (UEFI)**:
  - `yuna.conf`: Yuna OS (%ARCH%, UEFI) - Cambodia Anime Edition
  - `yuna-ram.conf`: Yuna OS (%ARCH%, UEFI, Copy to RAM)
  - `yuna-nomodeset.conf`: Yuna OS (%ARCH%, UEFI, Safe Graphics nomodeset)

---

## 3. Repository Architecture

```
yuna-os/
├── distro/                          # Archiso live distribution profile
│   ├── airootfs/                    # Root filesystem overlay for the live ISO
│   │   ├── etc/
│   │   │   ├── os-release           # Yuna OS distro branding
│   │   │   ├── issue & motd         # Login messages and live credentials
│   │   │   ├── pacman.d/mirrorlist  # Singapore & global HTTPS mirrors
│   │   │   ├── sudoers.d/           # Live user privilege rules
│   │   │   └── systemd/system/      # Autologin getty service override
│   │   ├── root/customize_airootfs.sh # Live bootstrap script
│   │   └── usr/local/bin/           # Helper scripts (yuna-welcome, yuna-mirrors)
│   ├── efiboot/loader/              # UEFI systemd-boot configuration & entries
│   ├── packages.x86_64              # Base live package manifest (fonts, TLS, firewall)
│   ├── pacman.conf                  # Pacman build configuration with [yuna] repo
│   ├── profiledef.sh                # Archiso image metadata and permissions
│   └── syslinux/                    # BIOS syslinux boot menu themes & entries
├── kernel/                          # Bare-metal x86_64 Rust kernel
│   ├── src/                         # Kernel entry (32-bit stub -> 64-bit long mode), VGA, COM1, shell
│   ├── tools/repack_elf32.py        # Multiboot container adapter
│   └── run.sh                       # QEMU launcher
├── packages/                        # Arch PKGBUILD sources
│   ├── yuna-tools/                  # Builds yuna-fetch, yunainstall, yuna-wifi
│   └── yuna-release/                # Distro branding, os-release hook
├── rust/                            # Rust workspace
│   ├── yuna-fetch/                  # Lightweight CLI system info tool
│   ├── yuna-wifi/                   # Interactive console Wi-Fi manager
│   └── yunainstall/                 # Guided CLI installer
├── scripts/                         # Build & validation automation
│   ├── build-repo.sh                # Builds local [yuna] pacman repository
│   ├── build-iso.sh                 # Builds bootable hybrid ISO image
│   └── check-profile.sh             # Static validation of profile and package availability
├── test.sh                          # Non-root test suite covering all components
└── Makefile                         # Build targets
```

---

## 4. Development & Build Workflow

### Prerequisites
On an Arch Linux system or container:
```sh
sudo pacman -S --needed base-devel git rust cargo python archiso qemu-system-x86
```

### Running the Test Suite
The test suite runs without root privileges and validates the Rust tools, installer logic, packaging scripts, archiso profile, and boots the bare-metal kernel in QEMU:
```sh
./test.sh
```

Fast check (skips kernel QEMU boot and package compilation):
```sh
./test.sh --fast
```

### Compiling Rust Tooling Locally
```sh
cd rust
cargo build --release
```

### Building the Package Repository
```sh
./scripts/build-repo.sh
# Outputs:
#   out/repo/yuna-tools-*.pkg.tar.zst
#   out/repo/yuna-release-*.pkg.tar.zst
#   out/repo/yuna.db.tar.gz
```

### Building the Bootable Live ISO
```sh
./scripts/build-repo.sh
sudo ./scripts/build-iso.sh
# Outputs:
#   out/yuna-<date>-x86_64.iso
```

### Testing the Live ISO in QEMU
```sh
qemu-system-x86_64 -enable-kvm -m 4096 -cdrom out/yuna-*.iso -boot d
```

---

## 5. Guided Installer (`yunainstall`)

To test the installer without modifying partitions:
```sh
yunainstall --dry-run --config examples/yuna.conf
```

To run the installation on target hardware:
```sh
sudo yunainstall
```

`yunainstall` automatically:
1. Detects UEFI or BIOS firmware.
2. Partitions the drive with GPT (`sgdisk`).
3. Sets up optional LUKS2 disk encryption (`cryptsetup`).
4. Formats filesystems (`ext4`, `btrfs`, `xfs`).
5. Runs `pacstrap` with base packages, font stack, TLS utilities, and UFW firewall.
6. Generates `/root/yuna-post.sh` to configure timezone, locale, users, passwords, systemd services, and bootloader (`systemd-boot` or `grub`).
7. Chroots and seals the newly installed operating system.

---

## 6. Author Note

Made with ❤️ by **Rikixz** for the **Cambodia community and anime fans**.  
Feedback, contributions, and bug reports are welcome on [GitHub](https://github.com/Rikixz-0night/yuna-os).
