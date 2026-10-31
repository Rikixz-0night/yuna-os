//! Block-device discovery through /sys and /proc (no external tools).

use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Partition {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct Disk {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub model: String,
    pub removable: bool,
    pub mounted: bool,
    pub partitions: Vec<Partition>,
}

impl Disk {
    pub fn size_human(&self) -> String {
        human_bytes(self.size_bytes)
    }
}

pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn read_trim(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn mounted_devices() -> Vec<String> {
    let mut devices = Vec::new();
    if let Ok(text) = fs::read_to_string("/proc/mounts") {
        for line in text.lines() {
            if let Some(source) = line.split_whitespace().next() {
                if let Some(name) = source.strip_prefix("/dev/") {
                    devices.push(name.to_string());
                }
            }
        }
    }
    devices
}

pub fn list() -> Vec<Disk> {
    let mounted = mounted_devices();
    let mut disks = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/block") else {
        return disks;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip virtual and optical devices.
        let interesting = ["sd", "nvme", "vd", "mmcblk", "xvd", "hd"]
            .iter()
            .any(|prefix| name.starts_with(prefix));
        if !interesting || name.starts_with("sr") {
            continue;
        }

        let base = format!("/sys/block/{name}");
        let sectors: u64 = read_trim(&format!("{base}/size"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if sectors == 0 {
            continue;
        }

        let model = read_trim(&format!("{base}/device/model"))
            .or_else(|| read_trim(&format!("{base}/device/name")))
            .unwrap_or_else(|| "unknown model".to_string());
        let removable = read_trim(&format!("{base}/removable"))
            .map(|v| v == "1")
            .unwrap_or(false);
        let is_mounted = mounted.iter().any(|device| device == &name || device.starts_with(&name));

        let mut partitions = Vec::new();
        if let Ok(children) = fs::read_dir(&base) {
            for child in children.flatten() {
                let child_name = child.file_name().to_string_lossy().to_string();
                if !child_name.starts_with(&name) {
                    continue;
                }
                if !Path::new(&format!("{base}/{child_name}/partition")).exists() {
                    continue;
                }
                let part_sectors: u64 = read_trim(&format!("{base}/{child_name}/size"))
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                partitions.push(Partition {
                    name: child_name.clone(),
                    path: format!("/dev/{child_name}"),
                    size_bytes: part_sectors * 512,
                });
            }
        }
        partitions.sort_by(|a, b| a.name.cmp(&b.name));

        disks.push(Disk {
            name: name.clone(),
            path: format!("/dev/{name}"),
            size_bytes: sectors * 512,
            model,
            removable,
            mounted: is_mounted,
            partitions,
        });
    }

    disks.sort_by(|a, b| a.name.cmp(&b.name));
    disks
}

pub fn is_uefi() -> bool {
    Path::new("/sys/firmware/efi").exists()
}

/// Partition device paths for a freshly created GPT layout.
pub fn partition_path(disk: &str, index: u8) -> String {
    let separator = if disk.starts_with("/dev/nvme")
        || disk.starts_with("/dev/mmcblk")
        || disk.starts_with("/dev/loop")
    {
        "p"
    } else {
        ""
    };
    format!("{disk}{separator}{index}")
}
