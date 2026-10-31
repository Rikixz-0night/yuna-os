//! yunainstall - guided installer for Yuna OS (Arch base).
//!
//! Interactive:
//!   sudo yunainstall
//!
//! Unattended:
//!   sudo yunainstall --config /etc/yunainstall.conf --yes
//!
//! Preview without touching disks:
//!   yunainstall --dry-run

mod disks;
mod exec;
mod install;
mod plan;
mod profiles;
mod ui;

use std::process::Command;

use exec::Runner;
use plan::{
    Bootloader, Desktop, Drivers, Filesystem, Kernel, Plan, Swap,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    ui::init_color();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut config: Option<String> = None;
    let mut dry_run = false;
    let mut assume_yes = false;
    let mut print_script = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--list-disks" | "-l" => {
                list_disks();
                return;
            }
            "--config" | "-c" => {
                index += 1;
                if index >= args.len() {
                    fail("--config needs a file path");
                }
                config = Some(args[index].clone());
            }
            "--dry-run" | "-n" => dry_run = true,
            "--print-script" => print_script = true,
            "--yes" | "-y" => assume_yes = true,
            "--example-config" => {
                print!("{}", Plan::example_config());
                return;
            }
            "--version" | "-V" => {
                println!("yunainstall {VERSION}");
                return;
            }
            "--help" | "-h" => {
                help();
                return;
            }
            other => fail(&format!("unknown option `{other}` (try --help)")),
        }
        index += 1;
    }

    if !print_script {
        ui::banner();
    }

    let plan = match config {
        Some(path) => {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| fail(&format!("cannot read {path}: {error}")));
            Plan::from_config(&text).unwrap_or_else(|error| fail(&error))
        }
        None => interactive(),
    };

    if let Err(error) = plan.validate() {
        fail(&error);
    }

    if print_script {
        let crypt = if plan.encrypt { Some("CRYPT-UUID") } else { None };
        print!("{}", install::post_script(&plan, "ROOT-UUID", crypt));
        return;
    }

    print_summary(&plan);

    if !dry_run {
        if !is_root() {
            fail("installer must run as root (try: sudo yunainstall)");
        }
        if let Some(disk) = disks::list().into_iter().find(|d| d.path == plan.disk) {
            if disk.mounted {
                fail(&format!(
                    "{} has mounted partitions; unmount them first",
                    plan.disk
                ));
            }
        } else {
            fail(&format!("{} is not a known block device", plan.disk));
        }
    }

    if !assume_yes && !dry_run {
        if !ui::confirm("write this plan to disk and install", false) {
            println!("{}", ui::yellow("aborted, nothing was changed"));
            std::process::exit(1);
        }
    }

    if dry_run {
        println!();
        println!("{}", ui::yellow("dry run: commands are printed, nothing is executed"));
        println!();
    }

    let runner = Runner::new(dry_run);
    match install::run(&plan, &runner) {
        Ok(()) => {}
        Err(error) => fail(&error),
    }
}

fn interactive() -> Plan {
    let mut plan = Plan::defaults();

    ui::heading("Target disk");
    let available: Vec<disks::Disk> = disks::list();
    if available.is_empty() {
        fail("no block devices found");
    }

    let labels: Vec<String> = available
        .iter()
        .map(|disk| {
            let flags = [
                disk.mounted.then_some("mounted"),
                disk.removable.then_some("removable"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(", ");
            let suffix = if flags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", flags)
            };
            format!("{}  {}  {}{}", disk.path, disk.size_human(), disk.model, suffix)
        })
        .collect();

    let default_disk = available
        .iter()
        .position(|d| !d.mounted && !d.removable)
        .unwrap_or(0);
    let chosen = ui::select("choose a disk (all data will be destroyed)", &labels, default_disk);
    plan.disk = available[chosen].path.clone();

    ui::heading("System identity");
    plan.hostname = ui::ask("hostname", &plan.hostname);
    plan.username = ui::ask("username", &plan.username);

    loop {
        let first = ui::ask_secret("password");
        if first.is_empty() {
            println!("{}", ui::red("password cannot be empty"));
            continue;
        }
        let second = ui::ask_secret("confirm password");
        if first != second {
            println!("{}", ui::red("passwords do not match"));
            continue;
        }
        plan.password = first;
        break;
    }

    if !ui::confirm("use the same password for root", true) {
        let root = ui::ask_secret("root password");
        plan.root_password = if root.is_empty() { None } else { Some(root) };
    }

    ui::heading("Locale and keyboard");
    loop {
        let timezone = ui::ask("timezone", &plan.timezone);
        if std::path::Path::new(&format!("/usr/share/zoneinfo/{timezone}")).exists() {
            plan.timezone = timezone;
            break;
        }
        println!("{}", ui::red(&format!("unknown timezone `{timezone}`")));
    }
    plan.locale = ui::ask("locale", &plan.locale);
    plan.keymap = ui::ask("keymap", &plan.keymap);

    ui::heading("Storage");
    plan.filesystem = choose(
        "filesystem",
        &[
            ("btrfs - modern CoW with @ and @home subvolumes (Recommended)", Filesystem::Btrfs),
            ("ext4  - rock-solid classic journaling filesystem", Filesystem::Ext4),
            ("f2fs  - Flash-Friendly File System (optimized for SSD/NVMe)", Filesystem::F2fs),
            ("xfs   - high performance for large enterprise files", Filesystem::Xfs),
        ],
        0,
    );
    plan.encrypt = ui::confirm("encrypt the root partition with LUKS2", false);
    if plan.uefi {
        let size = ui::ask("EFI system partition size (MiB)", &plan.esp_mib.to_string());
        plan.esp_mib = size.parse().unwrap_or(plan.esp_mib);
    }

    ui::heading("Software");
    plan.kernel = choose(
        "kernel",
        &[
            ("linux     - latest stable vanilla Arch kernel (Recommended)", Kernel::Linux),
            ("linux-lts - long term support for rock-solid stability", Kernel::Lts),
            ("linux-zen - desktop tuned with low-latency responsiveness", Kernel::Zen),
        ],
        0,
    );
    plan.desktop = choose(
        "desktop",
        &[
            ("kde      - Plasma 6 (Recommended for modern full-featured desktop)", Desktop::Kde),
            ("hyprland - dynamic Wayland tiling compositor (modern & fluid)", Desktop::Hyprland),
            ("xfce     - ultra-lightweight classic desktop (fast, minimal RAM)", Desktop::Xfce),
            ("gnome    - modern streamlined GNOME Shell", Desktop::Gnome),
            ("sway     - i3-compatible Wayland tiling window manager", Desktop::Sway),
            ("cinnamon - traditional sleek desktop", Desktop::Cinnamon),
            ("none     - minimal console only", Desktop::None),
        ],
        0,
    );

    let detected = profiles::detect_gpus();
    let detected_label = if detected.is_empty() {
        "no GPU detected".to_string()
    } else {
        detected
            .iter()
            .map(|vendor| vendor.name())
            .collect::<Vec<_>>()
            .join(" + ")
    };
    println!(
        "{}",
        ui::dim(&format!(
            "graphics detected: {detected_label} (mesa is always installed with a desktop)"
        ))
    );
    plan.drivers = choose(
        "graphics drivers",
        &[
            ("auto    - open-source drivers (mesa + modesetting) (Recommended)", Drivers::Auto),
            ("nvidia  - proprietary nvidia-dkms for GeForce/RTX cards", Drivers::Nvidia),
            ("amd     - mesa + vulkan-radeon", Drivers::Amd),
            ("intel   - mesa + vulkan-intel", Drivers::Intel),
            ("none    - minimal mesa fallback only", Drivers::None),
        ],
        0,
    );

    plan.swap = choose(
        "swap",
        &[
            ("zram - compressed in-RAM swap (Recommended, zero disk wear)", Swap::Zram),
            ("none - no swap", Swap::None),
        ],
        0,
    );

    ui::heading("Boot");
    plan.uefi = disks::is_uefi();
    if plan.uefi {
        plan.bootloader = choose(
            "bootloader",
            &[
                ("systemd-boot - simple, native UEFI bootloader (Recommended)", Bootloader::SystemdBoot),
                ("grub         - universal bootloader with BIOS & multi-boot support", Bootloader::Grub),
            ],
            0,
        );
    } else {
        println!("{}", ui::dim("BIOS firmware detected; grub will be used"));
        plan.bootloader = Bootloader::Grub;
    }

    plan
}

fn choose<T: Copy>(title: &str, options: &[(&str, T)], default: usize) -> T {
    let labels: Vec<String> = options.iter().map(|(label, _)| label.to_string()).collect();
    let index = ui::select(title, &labels, default);
    options[index].1
}

fn print_summary(plan: &Plan) {
    ui::heading("Install plan");
    let width = plan
        .summary()
        .iter()
        .map(|(key, _)| key.len())
        .max()
        .unwrap_or(8);
    for (key, value) in plan.summary() {
        println!("  {} {}", ui::cyan(&format!("{key:<width$}")), value);
    }
}

fn list_disks() {
    ui::heading("Block devices");
    let found = disks::list();
    if found.is_empty() {
        println!("  no block devices found");
        return;
    }

    for disk in found {
        let flags = [
            disk.mounted.then_some("mounted"),
            disk.removable.then_some("removable"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");
        println!(
            "  {}  {:<9}  {}{}",
            ui::cyan(&disk.path),
            disk.size_human(),
            disk.model,
            if flags.is_empty() { String::new() } else { format!("  [{}]", flags) }
        );
        for partition in disk.partitions {
            println!(
                "      {}  {}",
                partition.path,
                disks::human_bytes(partition.size_bytes)
            );
        }
    }
    println!();
    println!("{}", ui::dim("firmware: ") .to_string() + if disks::is_uefi() { "UEFI" } else { "BIOS" });
}

fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false)
}

fn help() {
    println!("yunainstall {VERSION} - Yuna OS installer");
    println!();
    println!("usage: yunainstall [options]");
    println!();
    println!("  -l, --list-disks       list block devices and firmware mode");
    println!("  -c, --config <file>    unattended install from a key=value plan");
    println!("  -n, --dry-run          print every command without executing it");
    println!("  -y, --yes              skip the final confirmation");
    println!("      --print-script     print the generated chroot script and exit");
    println!("      --example-config   print a template plan");
    println!("  -V, --version          print version");
    println!("  -h, --help             this text");
}

fn fail(message: &str) -> ! {
    eprintln!("{} {}", ui::red("error:"), message);
    std::process::exit(1);
}
