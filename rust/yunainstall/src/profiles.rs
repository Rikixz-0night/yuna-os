//! Package sets per install profile.
//!
//! Everything comes from the official Arch repositories (`core`, `extra`) plus
//! the small offline `[yuna]` repository. The installer never invents package
//! names: every entry here is a real Arch package, so `pacman` keeps working
//! like a normal Arch system afterwards.

use crate::plan::{Desktop, Drivers, Kernel, Plan};

pub fn cpu_microcode() -> Option<&'static str> {
    let text = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    if text.contains("GenuineIntel") {
        Some("intel-ucode")
    } else if text.contains("AuthenticAMD") {
        Some("amd-ucode")
    } else {
        None
    }
}

/// Graphics vendors present in /sys, used when `drivers=auto`.
pub fn detect_gpus() -> Vec<Drivers> {
    let mut found: Vec<Drivers> = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") else {
        return found;
    };

    for entry in entries.flatten() {
        let base = entry.path();
        let class = read_trim(&base.join("class")).unwrap_or_default();
        let class = class.trim_start_matches("0x");
        // 0x0300 VGA, 0x0302 3D controller, 0x0380 other display.
        if !(class.starts_with("0300") || class.starts_with("0302") || class.starts_with("0380")) {
            continue;
        }

        let vendor = read_trim(&base.join("vendor")).unwrap_or_default();
        let driver = match vendor.trim_start_matches("0x") {
            "8086" => Drivers::Intel,
            "1002" => Drivers::Amd,
            "10de" => Drivers::Nvidia,
            _ => continue,
        };
        if !found.contains(&driver) {
            found.push(driver);
        }
    }

    found
}

fn read_trim(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn desktop_packages(desktop: Desktop) -> Vec<&'static str> {
    match desktop {
        Desktop::None => vec![],
        Desktop::Kde => vec![
            "plasma-meta",
            "sddm",
            "konsole",
            "dolphin",
            "ark",
            "spectacle",
            "kde-graphics-meta",
        ],
        Desktop::Gnome => vec!["gnome", "gnome-tweaks", "gdm"],
        Desktop::Xfce => vec![
            "xfce4-session",
            "xfwm4",
            "xfdesktop",
            "xfce4-panel",
            "xfce4-settings",
            "xfce4-terminal",
            "xfce4-appfinder",
            "thunar",
            "thunar-volman",
            "tumbler",
            "xfce4-power-manager",
            "xfce4-pulseaudio-plugin",
            "xfce4-whiskermenu-plugin",
            "xfce4-notifyd",
            "pavucontrol",
            "lightdm",
            "lightdm-gtk-greeter",
        ],
        Desktop::Hyprland => vec![
            "hyprland",
            "xdg-desktop-portal-hyprland",
            "xdg-desktop-portal-gtk",
            "kitty",
            "waybar",
            "wofi",
            "polkit-gnome",
            "greetd",
            "greetd-tuigreet",
            "noto-fonts",
            "ttf-jetbrains-mono-nerd",
        ],
        Desktop::Sway => vec![
            "sway",
            "swaylock",
            "swayidle",
            "waybar",
            "wofi",
            "foot",
            "xdg-desktop-portal-wlr",
            "xdg-desktop-portal-gtk",
            "polkit-gnome",
            "greetd",
            "greetd-tuigreet",
        ],
        Desktop::Cinnamon => vec![
            "cinnamon",
            "nemo",
            "lightdm",
            "lightdm-gtk-greeter",
        ],
    }
}

/// Audio, Bluetooth, fonts and XDG helpers shared by every desktop.
fn desktop_extras() -> Vec<&'static str> {
    vec![
        "pipewire",
        "pipewire-alsa",
        "pipewire-pulse",
        "pipewire-jack",
        "wireplumber",
        "pavucontrol",
        "alsa-utils",
        "bluez",
        "bluez-utils",
        "noto-fonts",
        "noto-fonts-emoji",
        "ttf-dejavu",
        "xdg-utils",
        "xdg-user-dirs",
    ]
}

/// GPU drivers for the detected/explicit vendor.
pub fn graphics_packages(plan: &Plan) -> Vec<&'static str> {
    let mut list: Vec<&'static str> = Vec::new();

    if plan.desktop != Desktop::None {
        list.push("mesa");
    }

    let vendors = match plan.drivers {
        // `auto` installs only open-source drivers. Proprietary NVIDIA is an
        // explicit choice, exactly like archinstall's "Graphics Drivers" step.
        Drivers::Auto => detect_gpus()
            .into_iter()
            .filter(|vendor| *vendor != Drivers::Nvidia)
            .collect(),
        Drivers::None => vec![],
        explicit => vec![explicit],
    };

    for vendor in vendors {
        match vendor {
            Drivers::Intel => {
                list.extend(["vulkan-intel", "intel-media-driver", "libva-intel-driver"]);
            }
            Drivers::Amd => {
                list.extend(["vulkan-radeon", "libva-mesa-driver"]);
            }
            Drivers::Nvidia => {
                list.extend(["nvidia-dkms", "nvidia-utils", "nvidia-settings", "dkms"]);
                list.push(match plan.kernel {
                    Kernel::Linux => "linux-headers",
                    Kernel::Lts => "linux-lts-headers",
                    Kernel::Zen => "linux-zen-headers",
                });
            }
            Drivers::Auto | Drivers::None => {}
        }
    }

    list
}

pub fn uses_nvidia(plan: &Plan) -> bool {
    plan.drivers == Drivers::Nvidia
}

/// Kernel argument needed for the proprietary driver on Wayland/Xorg.
pub fn nvidia_kernel_args(plan: &Plan) -> &'static str {
    if uses_nvidia(plan) {
        "nvidia_drm.modeset=1"
    } else {
        ""
    }
}

pub fn packages(plan: &Plan) -> Vec<String> {
    let mut list: Vec<String> = vec![
        "base".into(),
        "archlinux-keyring".into(),
        "linux-firmware".into(),
        "mkinitcpio".into(),
        "sudo".into(),
        "networkmanager".into(),
        "wpa_supplicant".into(),
        "iw".into(),
        "iwd".into(),
        "wireless-regdb".into(),
        "usbutils".into(),
        "ethtool".into(),
        "nano".into(),
        "vim".into(),
        "less".into(),
        "man-db".into(),
        "man-pages".into(),
        "git".into(),
        "rsync".into(),
        "bash-completion".into(),
        "fastfetch".into(),
        "pacman-contrib".into(),
        "reflector".into(),
        "dosfstools".into(),
        "mtools".into(),
        "gptfdisk".into(),
        "pciutils".into(),
        "zram-generator".into(),
        "ca-certificates".into(),
        "ca-certificates-utils".into(),
        "openssl".into(),
        "curl".into(),
        "wget".into(),
        "ufw".into(),
        "iptables".into(),
        "noto-fonts".into(),
        "noto-fonts-cjk".into(),
        "noto-fonts-emoji".into(),
        "ttf-dejavu".into(),
        "ttf-liberation".into(),
        // Disk & filesystem support
        "f2fs-tools".into(),
        "ntfs-3g".into(),
        "exfatprogs".into(),
        "smartmontools".into(),
        "nvme-cli".into(),
        // Hardware & guest drivers
        "sof-firmware".into(),
        "alsa-firmware".into(),
        "alsa-utils".into(),
        "libinput".into(),
        "xf86-input-libinput".into(),
        "qemu-guest-agent".into(),
        "yuna-tools".into(),
        "yuna-release".into(),
    ];

    list.push(plan.kernel.package().into());
    list.push(plan.filesystem.package().into());

    if let Some(ucode) = cpu_microcode() {
        list.push(ucode.into());
    }

    match plan.bootloader {
        crate::plan::Bootloader::SystemdBoot => list.push("efibootmgr".into()),
        crate::plan::Bootloader::Grub => {
            list.push("grub".into());
            if plan.uefi {
                list.push("efibootmgr".into());
            }
        }
    }

    if plan.encrypt {
        list.push("cryptsetup".into());
    }

    for package in desktop_packages(plan.desktop) {
        list.push(package.into());
    }
    if plan.desktop != Desktop::None {
        for package in desktop_extras() {
            list.push(package.into());
        }
    }
    for package in graphics_packages(plan) {
        list.push(package.into());
    }

    // Keep the list stable and free of duplicates (pacstrap handles duplicates,
    // but a clean plan is easier to read).
    let mut seen = std::collections::HashSet::new();
    list.retain(|package| seen.insert(package.clone()));
    list
}

pub fn display_manager(desktop: Desktop) -> Option<&'static str> {
    match desktop {
        Desktop::None => None,
        Desktop::Kde => Some("sddm"),
        Desktop::Gnome => Some("gdm"),
        Desktop::Xfce => Some("lightdm"),
        Desktop::Hyprland => Some("greetd"),
        Desktop::Sway => Some("greetd"),
        Desktop::Cinnamon => Some("lightdm"),
    }
}

pub fn kernel_name(kernel: Kernel) -> &'static str {
    kernel.package()
}
