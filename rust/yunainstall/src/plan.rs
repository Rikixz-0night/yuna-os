//! Install plan: choices, parsing and validation.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filesystem {
    Ext4,
    Btrfs,
    Xfs,
}

impl Filesystem {
    pub fn name(self) -> &'static str {
        match self {
            Filesystem::Ext4 => "ext4",
            Filesystem::Btrfs => "btrfs",
            Filesystem::Xfs => "xfs",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "ext4" => Some(Filesystem::Ext4),
            "btrfs" => Some(Filesystem::Btrfs),
            "xfs" => Some(Filesystem::Xfs),
            _ => None,
        }
    }

    pub fn tool(self) -> &'static str {
        match self {
            Filesystem::Ext4 => "mkfs.ext4",
            Filesystem::Btrfs => "mkfs.btrfs",
            Filesystem::Xfs => "mkfs.xfs",
        }
    }

    pub fn package(self) -> &'static str {
        match self {
            Filesystem::Ext4 => "e2fsprogs",
            Filesystem::Btrfs => "btrfs-progs",
            Filesystem::Xfs => "xfsprogs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kernel {
    Linux,
    Lts,
    Zen,
}

impl Kernel {
    pub fn package(self) -> &'static str {
        match self {
            Kernel::Linux => "linux",
            Kernel::Lts => "linux-lts",
            Kernel::Zen => "linux-zen",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "linux" => Some(Kernel::Linux),
            "linux-lts" | "lts" => Some(Kernel::Lts),
            "linux-zen" | "zen" => Some(Kernel::Zen),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desktop {
    None,
    Kde,
    Gnome,
    Hyprland,
}

impl Desktop {
    pub fn name(self) -> &'static str {
        match self {
            Desktop::None => "none",
            Desktop::Kde => "kde",
            Desktop::Gnome => "gnome",
            Desktop::Hyprland => "hyprland",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "none" | "console" => Some(Desktop::None),
            "kde" | "plasma" => Some(Desktop::Kde),
            "gnome" => Some(Desktop::Gnome),
            "hyprland" | "hypr" => Some(Desktop::Hyprland),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bootloader {
    SystemdBoot,
    Grub,
}

impl Bootloader {
    pub fn name(self) -> &'static str {
        match self {
            Bootloader::SystemdBoot => "systemd-boot",
            Bootloader::Grub => "grub",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "systemd-boot" | "systemd" | "sd-boot" => Some(Bootloader::SystemdBoot),
            "grub" => Some(Bootloader::Grub),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swap {
    None,
    Zram,
}

impl Swap {
    pub fn name(self) -> &'static str {
        match self {
            Swap::None => "none",
            Swap::Zram => "zram",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Swap::None),
            "zram" => Some(Swap::Zram),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drivers {
    Auto,
    None,
    Intel,
    Amd,
    Nvidia,
}

impl Drivers {
    pub fn name(self) -> &'static str {
        match self {
            Drivers::Auto => "auto",
            Drivers::None => "none",
            Drivers::Intel => "intel",
            Drivers::Amd => "amd",
            Drivers::Nvidia => "nvidia",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "" | "auto" | "detect" => Some(Drivers::Auto),
            "none" | "off" | "vesa" => Some(Drivers::None),
            "intel" | "i915" => Some(Drivers::Intel),
            "amd" | "amdgpu" | "radeon" => Some(Drivers::Amd),
            "nvidia" | "proprietary" => Some(Drivers::Nvidia),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub disk: String,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub root_password: Option<String>,
    pub timezone: String,
    pub locale: String,
    pub keymap: String,
    pub filesystem: Filesystem,
    pub kernel: Kernel,
    pub desktop: Desktop,
    pub drivers: Drivers,
    pub bootloader: Bootloader,
    pub swap: Swap,
    pub encrypt: bool,
    pub esp_mib: u32,
    pub uefi: bool,
}

impl Plan {
    pub fn defaults() -> Self {
        let timezone = detect_timezone();
        Self {
            disk: String::new(),
            hostname: "yuna".to_string(),
            username: "yuna".to_string(),
            password: String::new(),
            root_password: None,
            timezone,
            locale: "en_US.UTF-8".to_string(),
            keymap: "us".to_string(),
            filesystem: Filesystem::Ext4,
            kernel: Kernel::Linux,
            desktop: Desktop::None,
            drivers: Drivers::Auto,
            bootloader: if crate::disks::is_uefi() {
                Bootloader::SystemdBoot
            } else {
                Bootloader::Grub
            },
            swap: Swap::Zram,
            encrypt: false,
            esp_mib: 512,
            uefi: crate::disks::is_uefi(),
        }
    }

    pub fn summary(&self) -> Vec<(&'static str, String)> {
        vec![
            ("disk", self.disk.clone()),
            ("firmware", if self.uefi { "UEFI".into() } else { "BIOS".into() }),
            ("hostname", self.hostname.clone()),
            ("user", self.username.clone()),
            ("root password", if self.root_password.is_some() { "set".into() } else { "same as user".into() }),
            ("timezone", self.timezone.clone()),
            ("locale", self.locale.clone()),
            ("keymap", self.keymap.clone()),
            ("filesystem", self.filesystem.name().to_string()),
            ("kernel", self.kernel.package().to_string()),
            ("desktop", self.desktop.name().to_string()),
            ("drivers", self.drivers.name().to_string()),
            ("bootloader", self.bootloader.name().to_string()),
            ("swap", self.swap.name().to_string()),
            ("encryption", if self.encrypt { "LUKS2".into() } else { "off".into() }),
            ("ESP size", format!("{} MiB", self.esp_mib)),
        ]
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();

        if !self.disk.starts_with("/dev/") || !self.disk.chars().all(|c| c.is_ascii_alphanumeric() || c == '/') {
            errors.push(format!("invalid disk path: {}", self.disk));
        }
        if !valid_hostname(&self.hostname) {
            errors.push(format!(
                "invalid hostname `{}` (letters, digits and '-', max 63 chars)",
                self.hostname
            ));
        }
        if !valid_username(&self.username) {
            errors.push(format!(
                "invalid username `{}` (must start with a letter or '_', letters/digits/_/-)",
                self.username
            ));
        }
        if self.password.is_empty() {
            errors.push("password must not be empty".into());
        }
        if self.timezone.is_empty()
            || !self
                .timezone
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '+'))
        {
            errors.push(format!("invalid timezone `{}`", self.timezone));
        }
        if self.locale.is_empty()
            || !self
                .locale
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '@' | '-' | '_'))
        {
            errors.push(format!("invalid locale `{}`", self.locale));
        }
        if self.keymap.is_empty() || !self.keymap.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            errors.push(format!("invalid keymap `{}`", self.keymap));
        }
        if self.esp_mib < 256 {
            errors.push("ESP size must be at least 256 MiB".into());
        }
        if self.bootloader == Bootloader::SystemdBoot && !self.uefi {
            errors.push("systemd-boot requires UEFI; use grub on BIOS systems".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    pub fn from_config(text: &str) -> Result<Self, String> {
        let mut plan = Plan::defaults();
        let mut seen_disk = false;

        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {}: expected key=value", number + 1));
            };
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match key {
                "disk" => {
                    plan.disk = value.to_string();
                    seen_disk = true;
                }
                "hostname" => plan.hostname = value.to_string(),
                "username" => plan.username = value.to_string(),
                "password" => plan.password = value.to_string(),
                "root_password" => {
                    plan.root_password = if value.is_empty() { None } else { Some(value.to_string()) }
                }
                "timezone" => plan.timezone = value.to_string(),
                "locale" => plan.locale = value.to_string(),
                "keymap" => plan.keymap = value.to_string(),
                "filesystem" => {
                    plan.filesystem = Filesystem::from_name(value)
                        .ok_or_else(|| format!("unknown filesystem `{value}` (ext4, btrfs, xfs)"))?
                }
                "kernel" => {
                    plan.kernel = Kernel::from_name(value)
                        .ok_or_else(|| format!("unknown kernel `{value}` (linux, linux-lts, linux-zen)"))?
                }
                "desktop" => {
                    plan.desktop = Desktop::from_name(value)
                        .ok_or_else(|| format!("unknown desktop `{value}` (none, kde, gnome, hyprland)"))?
                }
                "drivers" | "gpu" => {
                    plan.drivers = Drivers::from_name(value).ok_or_else(|| {
                        format!("unknown drivers `{value}` (auto, none, intel, amd, nvidia)")
                    })?
                }
                "bootloader" => {
                    plan.bootloader = Bootloader::from_name(value)
                        .ok_or_else(|| format!("unknown bootloader `{value}` (systemd-boot, grub)"))?
                }
                "swap" => {
                    plan.swap = Swap::from_name(value)
                        .ok_or_else(|| format!("unknown swap setting `{value}` (none, zram)"))?
                }
                "encrypt" => plan.encrypt = parse_bool(value)?,
                "esp_mib" => {
                    plan.esp_mib = value
                        .parse()
                        .map_err(|_| format!("invalid esp_mib `{value}`"))?
                }
                other => return Err(format!("line {}: unknown key `{other}`", number + 1)),
            }
        }

        if !seen_disk {
            return Err("config is missing `disk=`".into());
        }

        plan.uefi = crate::disks::is_uefi();
        plan.validate()?;
        Ok(plan)
    }

    pub fn example_config() -> &'static str {
        r#"# /etc/yunainstall.conf - unattended install plan
# WARNING: contains passwords. chmod 600.
disk=/dev/nvme0n1
hostname=yuna
username=yuna
password=change-me
# root_password=change-me-too
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
"#
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.summary() {
            writeln!(f, "{key:<14} {value}")?;
        }
        Ok(())
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        _ => Err(format!("invalid boolean `{value}`")),
    }
}

fn valid_hostname(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

fn valid_username(value: &str) -> bool {
    if value.is_empty() || value.len() > 32 {
        return false;
    }
    let mut chars = value.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_lowercase() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn detect_timezone() -> String {
    if let Ok(link) = std::fs::read_link("/etc/localtime") {
        let text = link.to_string_lossy();
        if let Some(index) = text.find("zoneinfo/") {
            let zone = &text[index + "zoneinfo/".len()..];
            if !zone.is_empty() && zone != "UTC" {
                return zone.to_string();
            }
        }
    }
    "Asia/Phnom_Penh".to_string()
}
