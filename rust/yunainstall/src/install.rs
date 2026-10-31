//! The actual installation pipeline.

use std::fmt::Write as _;

use crate::disks;
use crate::exec::Runner;
use crate::plan::{Bootloader, Desktop, Plan, Swap};
use crate::profiles;
use crate::ui;

pub fn run(plan: &Plan, runner: &Runner) -> Result<(), String> {
    let disk = plan.disk.clone();

    ui::heading("Partitioning");
    runner.run("wipefs", &args(&["-a", &disk]))?;
    runner.run("sgdisk", &args(&["-Z", &disk]))?;

    if plan.uefi {
        let esp = format!("1:0:+{}M", plan.esp_mib);
        runner.run(
            "sgdisk",
            &args(&["-n", &esp, "-t", "1:ef00", "-c", "1:YUNA_ESP", &disk]),
        )?;
    } else {
        runner.run(
            "sgdisk",
            &args(&["-n", "1:0:+1M", "-t", "1:ef02", "-c", "1:YUNA_BIOS", &disk]),
        )?;
    }
    runner.run(
        "sgdisk",
        &args(&["-n", "2:0:0", "-t", "2:8300", "-c", "2:YUNA_ROOT", &disk]),
    )?;
    runner.run("partprobe", &args(&[&disk]))?;
    if !runner.dry_run {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let esp_path = disks::partition_path(&disk, 1);
    let root_path = disks::partition_path(&disk, 2);

    ui::heading("Filesystems");

    if plan.uefi {
        runner.run("mkfs.fat", &args(&["-F32", "-n", "YUNA_ESP", &esp_path]))?;
    }

    let root_device = if plan.encrypt {
        runner.run(
            "cryptsetup",
            &args(&[
                "luksFormat",
                "--type",
                "luks2",
                "--batch-mode",
                "--pbkdf",
                "argon2id",
                &root_path,
            ]),
        )?;
        runner.run("cryptsetup", &args(&["open", &root_path, "yuna_crypt"]))?;
        "/dev/mapper/yuna_crypt".to_string()
    } else {
        root_path.clone()
    };

    runner.run(plan.filesystem.tool(), &filesystem_args(plan, &root_device))?;
    runner.run("mount", &args(&[&root_device, "/mnt"]))?;

    if plan.uefi {
        runner.run("mkdir", &args(&["-p", "/mnt/boot"]))?;
        runner.run("mount", &args(&[&esp_path, "/mnt/boot"]))?;
    }

    ui::heading("Base system");
    if !std::path::Path::new("/etc/pacman.d/gnupg/trustdb.gpg").exists() {
        // A stock Arch ISO already has this; the guard covers rescue shells.
        runner.run("pacman-key", &args(&["--init"]))?;
        runner.run("pacman-key", &args(&["--populate", "archlinux"]))?;
    }
    let packages = profiles::packages(plan);
    let mut pacstrap_args = vec!["/mnt".to_string()];
    pacstrap_args.extend(packages);
    runner.run("pacstrap", &pacstrap_args)?;

    // pacstrap creates an empty /etc/fstab; append the generated one.
    runner.shell("genfstab -U /mnt >> /mnt/etc/fstab")?;

    let root_uuid = runner.capture("blkid", &args(&["-s", "UUID", "-o", "value", &root_device]))?;
    let crypt_uuid = if plan.encrypt {
        Some(runner.capture("blkid", &args(&["-s", "UUID", "-o", "value", &root_path]))?)
    } else {
        None
    };

    ui::heading("Configuring the new system");
    let script = post_script(plan, &root_uuid, crypt_uuid.as_deref());
    runner.write("/mnt/root/yuna-post.sh", &script)?;
    runner.run("arch-chroot", &args(&["/mnt", "bash", "/root/yuna-post.sh"]))?;

    ui::heading("Finishing");
    runner.run("umount", &args(&["-R", "/mnt"]))?;

    println!();
    println!("{}", ui::green("installation finished"));
    println!(
        "{}",
        ui::dim("remove the installation medium and reboot into Yuna OS")
    );
    Ok(())
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn filesystem_args(plan: &Plan, device: &str) -> Vec<String> {
    let force = match plan.filesystem {
        crate::plan::Filesystem::Ext4 => "-F",
        crate::plan::Filesystem::Btrfs | crate::plan::Filesystem::Xfs => "-f",
    };
    args(&[force, "-L", "YUNA_ROOT", device])
}

fn b64(value: &str) -> String {
    // Minimal base64 encoder (RFC 4648) so passwords never need shell quoting.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = value.as_bytes();
    let mut out = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

pub fn post_script(plan: &Plan, root_uuid: &str, crypt_uuid: Option<&str>) -> String {
    let mut script = String::new();
    let kernel = profiles::kernel_name(plan.kernel);
    let root_password = plan.root_password.clone().unwrap_or_else(|| plan.password.clone());

    writeln!(script, "#!/usr/bin/env bash").unwrap();
    writeln!(script, "set -euo pipefail").unwrap();
    writeln!(script, "echo '[yuna] timezone and clock'").unwrap();
    writeln!(
        script,
        "ln -sf /usr/share/zoneinfo/{} /etc/localtime",
        plan.timezone
    )
    .unwrap();
    writeln!(script, "hwclock --systohc").unwrap();

    writeln!(script, "echo '[yuna] locale'").unwrap();
    writeln!(script, "echo '{} UTF-8' > /etc/locale.gen", plan.locale).unwrap();
    writeln!(script, "locale-gen").unwrap();
    writeln!(script, "echo 'LANG={}' > /etc/locale.conf", plan.locale).unwrap();
    writeln!(script, "echo 'KEYMAP={}' > /etc/vconsole.conf", plan.keymap).unwrap();

    writeln!(script, "echo '[yuna] identity'").unwrap();
    writeln!(script, "echo '{}' > /etc/hostname", plan.hostname).unwrap();
    writeln!(script, "cat > /etc/hosts <<'YUNA_HOSTS'").unwrap();
    writeln!(script, "127.0.0.1 localhost").unwrap();
    writeln!(script, "::1 localhost").unwrap();
    writeln!(
        script,
        "127.0.1.1 {0}.localdomain {0}",
        plan.hostname
    )
    .unwrap();
    writeln!(script, "YUNA_HOSTS").unwrap();

    writeln!(script, "echo '[yuna] passwords'").unwrap();
    writeln!(
        script,
        "root_pw=$(printf '%s' '{}' | base64 -d)",
        b64(&root_password)
    )
    .unwrap();
    writeln!(script, "printf 'root:%s\\n' \"$root_pw\" | chpasswd").unwrap();
    writeln!(
        script,
        "useradd -m -G wheel -s /bin/bash '{}'",
        plan.username
    )
    .unwrap();
    writeln!(
        script,
        "user_pw=$(printf '%s' '{}' | base64 -d)",
        b64(&plan.password)
    )
    .unwrap();
    writeln!(
        script,
        "printf '%s:%s\\n' '{}' \"$user_pw\" | chpasswd",
        plan.username
    )
    .unwrap();
    writeln!(
        script,
        "sed -i 's/^# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /etc/sudoers"
    )
    .unwrap();

    writeln!(script, "echo '[yuna] services'").unwrap();
    writeln!(script, "systemctl enable NetworkManager").unwrap();
    writeln!(script, "systemctl enable systemd-timesyncd").unwrap();
    writeln!(script, "systemctl enable fstrim.timer").unwrap();
    writeln!(script, "systemctl enable ufw.service || true").unwrap();
    writeln!(script, "sed -i 's/^#Color/Color/' /etc/pacman.conf").unwrap();
    writeln!(script, "sed -i 's/^#ParallelDownloads = 5/ParallelDownloads = 10/' /etc/pacman.conf").unwrap();

    writeln!(script, "echo '[yuna] pacman keyring and Arch mirrors'").unwrap();
    writeln!(script, "pacman-key --init || true").unwrap();
    writeln!(script, "pacman-key --populate archlinux || true").unwrap();
    writeln!(
        script,
        "if command -v reflector >/dev/null 2>&1; then reflector --country Singapore --protocol https --latest 20 --sort rate --save /etc/pacman.d/mirrorlist || reflector --protocol https --latest 20 --sort rate --save /etc/pacman.d/mirrorlist || true; fi"
    )
    .unwrap();

    if plan.desktop != Desktop::None {
        writeln!(
            script,
            "if [[ -f /usr/lib/systemd/system/bluetooth.service ]]; then systemctl enable bluetooth.service; fi"
        )
        .unwrap();
    }

    if plan.swap == Swap::Zram {
        writeln!(script, "cat > /etc/systemd/zram-generator.conf <<'YUNA_ZRAM'").unwrap();
        writeln!(script, "[zram0]").unwrap();
        writeln!(script, "zram-size = ram / 2").unwrap();
        writeln!(script, "compression-algorithm = zstd").unwrap();
        writeln!(script, "YUNA_ZRAM").unwrap();
    }

    if plan.encrypt {
        writeln!(script, "echo '[yuna] initramfs hooks'").unwrap();
        writeln!(
            script,
            "sed -i 's/^\\(HOOKS=(.*\\)block /\\1block encrypt /' /etc/mkinitcpio.conf"
        )
        .unwrap();
    }

    if profiles::uses_nvidia(plan) {
        writeln!(script, "echo '[yuna] nvidia kernel modules'").unwrap();
        writeln!(script, "dkms autoinstall || true").unwrap();
    }

    writeln!(script, "mkinitcpio -P").unwrap();

    writeln!(script, "echo '[yuna] bootloader'").unwrap();
    let nvidia_args = profiles::nvidia_kernel_args(plan);
    match plan.bootloader {
        Bootloader::SystemdBoot => {
            writeln!(script, "bootctl --esp-path=/boot install").unwrap();
            writeln!(script, "mkdir -p /boot/loader/entries").unwrap();
            writeln!(script, "cat > /boot/loader/loader.conf <<'YUNA_LOADER'").unwrap();
            writeln!(script, "default  yuna.conf").unwrap();
            writeln!(script, "timeout  3").unwrap();
            writeln!(script, "editor   no").unwrap();
            writeln!(script, "YUNA_LOADER").unwrap();
            writeln!(
                script,
                "cat > /boot/loader/entries/yuna.conf <<'YUNA_ENTRY'"
            )
            .unwrap();
            writeln!(script, "title   Yuna OS").unwrap();
            writeln!(script, "linux   /vmlinuz-{}", kernel).unwrap();
            writeln!(script, "initrd  /initramfs-{}.img", kernel).unwrap();
            let options = match crypt_uuid {
                Some(uuid) => format!(
                    "cryptdevice=UUID={uuid}:yuna_crypt root=/dev/mapper/yuna_crypt rw quiet"
                ),
                None => format!("root=UUID={root_uuid} rw quiet"),
            };
            let options = if nvidia_args.is_empty() {
                options
            } else {
                format!("{options} {nvidia_args}")
            };
            writeln!(script, "options {}", options).unwrap();
            writeln!(script, "YUNA_ENTRY").unwrap();
        }
        Bootloader::Grub => {
            if plan.uefi {
                writeln!(
                    script,
                    "grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=YUNA --recheck"
                )
                .unwrap();
            } else {
                writeln!(script, "grub-install --target=i386-pc --recheck '{}'", plan.disk).unwrap();
            }
            writeln!(script, "sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=3/' /etc/default/grub").unwrap();
            let kernel_args = match crypt_uuid {
                Some(uuid) => format!(
                    "cryptdevice=UUID={uuid}:yuna_crypt root=/dev/mapper/yuna_crypt rw quiet"
                ),
                None => "rw quiet".to_string(),
            };
            let kernel_args = if nvidia_args.is_empty() {
                kernel_args
            } else {
                format!("{kernel_args} {nvidia_args}")
            };
            writeln!(
                script,
                "sed -i 's|^GRUB_CMDLINE_LINUX=.*|GRUB_CMDLINE_LINUX=\"{}\"|' /etc/default/grub",
                kernel_args
            )
            .unwrap();
            writeln!(script, "grub-mkconfig -o /boot/grub/grub.cfg").unwrap();
        }
    }

    if let Some(manager) = profiles::display_manager(plan.desktop) {
        writeln!(script, "systemctl enable {}", manager).unwrap();
    }

    if plan.desktop == Desktop::Hyprland {
        writeln!(script, "mkdir -p /etc/greetd").unwrap();
        writeln!(
            script,
            "useradd -r -M -d /var/lib/greetd -s /usr/bin/nologin greeter 2>/dev/null || true"
        )
        .unwrap();
        writeln!(script, "install -d -o greeter -g greeter /var/lib/greetd").unwrap();
        writeln!(script, "cat > /etc/greetd/config.toml <<'YUNA_GREETD'").unwrap();
        writeln!(script, "[terminal]").unwrap();
        writeln!(script, "vt = 1").unwrap();
        writeln!(script, "[default_session]").unwrap();
        writeln!(script, "command = \"tuigreet --time --remember --cmd Hyprland\"").unwrap();
        writeln!(script, "user = \"greeter\"").unwrap();
        writeln!(script, "YUNA_GREETD").unwrap();
    }

    writeln!(script, "rm -f /root/yuna-post.sh").unwrap();
    writeln!(script, "echo '[yuna] done'").unwrap();
    script
}

#[allow(dead_code)]
fn ensure_root(runner: &Runner) -> Result<(), String> {
    let uid = runner.capture("id", &args(&["-u"]))?;
    if uid != "0" {
        return Err("yunainstall must run as root (try: sudo yunainstall)".into());
    }
    Ok(())
}
