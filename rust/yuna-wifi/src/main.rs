//! yuna-wifi - connect to Wi-Fi from the console.
//!
//! A small, dependency-free front-end for NetworkManager. It exists so a fresh
//! Yuna OS live session can join a wireless network with one command, without
//! remembering `nmcli` syntax:
//!
//!   yuna-wifi                      interactive: scan, pick, connect
//!   yuna-wifi --list               list visible networks
//!   yuna-wifi --status             show the current connection
//!   yuna-wifi --connect SSID       join a network (asks for the password)
//!   yuna-wifi --connect SSID --password KEY
//!   yuna-wifi --connect SSID --hidden
//!
//! Every network operation goes through the real NetworkManager daemon, so the
//! connection is immediately usable by pacman, the installer and the browser
//! once a desktop is installed.

use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

static COLOR: AtomicBool = AtomicBool::new(false);

fn color(code: &str, text: &str) -> String {
    if COLOR.load(Ordering::Relaxed) {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}

struct AccessPoint {
    ssid: String,
    signal: u8,
    security: String,
    in_use: bool,
}

impl AccessPoint {
    fn secured(&self) -> bool {
        !self.security.is_empty() && self.security != "--"
    }
}

fn main() {
    COLOR.store(io::stdout().is_terminal(), Ordering::Relaxed);

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut list = false;
    let mut status = false;
    let mut connect: Option<String> = None;
    let mut password: Option<String> = None;
    let mut hidden = false;
    let mut iface: Option<String> = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--list" | "-l" => list = true,
            "--status" | "-s" => status = true,
            "--connect" | "-c" => {
                index += 1;
                if index >= args.len() {
                    fail("--connect needs a network name");
                }
                connect = Some(args[index].clone());
            }
            "--password" | "-p" => {
                index += 1;
                if index >= args.len() {
                    fail("--password needs a value");
                }
                password = Some(args[index].clone());
            }
            "--hidden" => hidden = true,
            "--iface" | "-i" => {
                index += 1;
                if index >= args.len() {
                    fail("--iface needs a device name");
                }
                iface = Some(args[index].clone());
            }
            "--version" | "-V" => {
                println!("yuna-wifi {}", env!("CARGO_PKG_VERSION"));
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

    if !has_nmcli() {
        eprintln!(
            "{} NetworkManager (nmcli) was not found.\n\
             On the live medium it is installed; from a bare Arch install run:\n\
             \n  sudo pacman -S networkmanager\n  sudo systemctl enable --now NetworkManager\n",
            color(RED, "error:")
        );
        std::process::exit(1);
    }

    // Soft-blocked radios are a common reason a laptop "has no Wi-Fi".
    let _ = Command::new("rfkill").args(["unblock", "wifi"]).output();

    let device = iface.clone().or_else(wifi_device);

    if status {
        match device {
            Some(device) => print_status(&device),
            None => {
                println!("{}", color(YELLOW, "no wireless device detected"));
                print_devices();
            }
        }
        return;
    }

    if list {
        let Some(device) = device else {
            fail("no wireless device detected (is the firmware loaded?)");
        };
        match scan(&device) {
            Ok(points) => print_table(&points),
            Err(error) => fail(&error),
        }
        return;
    }

    if let Some(ssid) = connect {
        let Some(device) = device else {
            fail("no wireless device detected (is the firmware loaded?)");
        };
        let security = scan(&device)
            .ok()
            .and_then(|points| points.into_iter().find(|point| point.ssid == ssid))
            .map(|point| point.security)
            .unwrap_or_default();

        let password = match password {
            Some(value) => Some(value),
            None if hidden && io::stdin().is_terminal() => {
                let value = ask_secret("password (leave empty for an open network)");
                if value.is_empty() { None } else { Some(value) }
            }
            None if hidden => fail("hidden networks need --password"),
            None if security.is_empty() || security == "--" => None,
            None if io::stdin().is_terminal() => Some(ask_secret("password")),
            None => fail("this network needs a password: pass --password"),
        };

        match join(&device, &ssid, password.as_deref(), hidden) {
            Ok(()) => {
                println!("{} connected to {ssid}", color(GREEN, "ok:"));
                print_status(&device);
            }
            Err(error) => fail(&error),
        }
        return;
    }

    if !io::stdin().is_terminal() {
        help();
        return;
    }

    interactive(device);
}

fn help() {
    println!("yuna-wifi {} - console Wi-Fi for Yuna OS", env!("CARGO_PKG_VERSION"));
    println!();
    println!("usage: yuna-wifi [options]");
    println!();
    println!("  -l, --list                list visible networks");
    println!("  -s, --status              show the current connection");
    println!("  -c, --connect <ssid>      join a network");
    println!("  -p, --password <key>      password for --connect");
    println!("      --hidden              the network is hidden");
    println!("  -i, --iface <device>      wireless interface (default: auto)");
    println!("  -V, --version             print version");
    println!("  -h, --help                this text");
    println!();
    println!("Without options it opens an interactive scan-and-connect menu.");
    println!("Alternatives: {} (NetworkManager TUI), {} (iwd).", color(CYAN, "nmtui"), color(CYAN, "iwctl"));
}

fn interactive(device: Option<String>) {
    println!();
    println!("{}", color(BOLD, "  Yuna OS  ·  Wi-Fi"));
    println!();

    let Some(device) = device else {
        println!(
            "{}",
            color(RED, "  no wireless device detected — check `ip link` and rfkill")
        );
        print_devices();
        return;
    };

    println!("  interface {}  {}", color(CYAN, &device), color(DIM, "(scanning...)"));
    let points = match scan(&device) {
        Ok(points) if !points.is_empty() => points,
        Ok(_) => {
            println!("{}", color(YELLOW, "  no networks in range"));
            return;
        }
        Err(error) => {
            println!("{} {error}", color(RED, "  scan failed:"));
            return;
        }
    };

    print_table(&points);

    loop {
        print!(
            "\n  select a network {} or {}: ",
            color(DIM, "[1-{}]"),
            color(DIM, "q to quit")
        );
        io::stdout().flush().ok();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return;
        }
        let line = line.trim();
        if line.is_empty() || line.eq_ignore_ascii_case("q") {
            return;
        }

        let Some(point) = line
            .parse::<usize>()
            .ok()
            .and_then(|choice| points.get(choice.wrapping_sub(1)))
        else {
            println!("{}", color(RED, "  invalid choice"));
            continue;
        };

        let password = if point.secured() {
            let value = ask_secret(&format!("password for {}", point.ssid));
            if value.is_empty() {
                println!("{}", color(RED, "  password cannot be empty"));
                continue;
            }
            Some(value)
        } else {
            None
        };

        println!("  connecting to {} ...", color(BOLD, &point.ssid));
        match join(&device, &point.ssid, password.as_deref(), false) {
            Ok(()) => {
                println!("{} connected", color(GREEN, "ok:"));
                print_status(&device);
                return;
            }
            Err(error) => {
                println!("{} {error}", color(RED, "failed:"));
                if !point.secured() {
                    if let Some(value) = ask_optional_secret("password (leave empty for open network)") {
                        match join(&device, &point.ssid, Some(&value), false) {
                            Ok(()) => {
                                println!("{} connected", color(GREEN, "ok:"));
                                print_status(&device);
                                return;
                            }
                            Err(error) => println!("{} {error}", color(RED, "failed:")),
                        }
                    }
                }
                if !confirm("try another network", true) {
                    return;
                }
            }
        }
    }
}

fn has_nmcli() -> bool {
    Command::new("nmcli")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn nmcli(args: &[&str]) -> Result<String, String> {
    let output = Command::new("nmcli")
        .args(args)
        .output()
        .map_err(|error| format!("cannot run nmcli: {error}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stderr.is_empty() {
            Err(if stdout.is_empty() { "nmcli failed".into() } else { stdout })
        } else {
            Err(stderr)
        }
    }
}

fn wifi_device() -> Option<String> {
    // NetworkManager may still be enumerating devices right after login.
    for _ in 0..10 {
        if let Some(device) = wifi_device_once() {
            return Some(device);
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    None
}

fn wifi_device_once() -> Option<String> {
    let text = nmcli(&["-t", "-f", "DEVICE,TYPE,STATE", "device", "status"]).ok()?;
    for line in text.lines() {
        let fields = split_terse(line);
        if fields.len() >= 3 && fields[1] == "wifi" {
            return Some(fields[0].clone());
        }
    }
    None
}

fn scan(device: &str) -> Result<Vec<AccessPoint>, String> {
    // A rescan is best-effort: NetworkManager throttles it, and the cached scan
    // is still useful.
    let _ = nmcli(&["device", "wifi", "rescan", "ifname", device]);

    let text = nmcli(&[
        "-t",
        "-f",
        "IN-USE,SSID,SIGNAL,SECURITY",
        "device",
        "wifi",
        "list",
        "ifname",
        device,
    ])?;

    let mut points: Vec<AccessPoint> = Vec::new();
    for line in text.lines() {
        let fields = split_terse(line);
        if fields.len() < 4 {
            continue;
        }
        let ssid = fields[1].clone();
        if ssid.is_empty() {
            continue;
        }
        let signal: u8 = fields[2].parse().unwrap_or(0);
        let in_use = fields[0].contains('*');

        if let Some(existing) = points.iter_mut().find(|point| point.ssid == ssid) {
            existing.in_use |= in_use;
            if signal > existing.signal {
                existing.signal = signal;
                existing.security = fields[3].clone();
            }
        } else {
            points.push(AccessPoint {
                ssid,
                signal,
                security: fields[3].clone(),
                in_use,
            });
        }
    }

    points.sort_by(|a, b| b.signal.cmp(&a.signal).then_with(|| a.ssid.cmp(&b.ssid)));
    Ok(points)
}

fn print_table(points: &[AccessPoint]) {
    if points.is_empty() {
        println!("  no networks in range");
        return;
    }

    println!();
    println!(
        "  {}  {:<12} {:<9} {}",
        color(DIM, "# "),
        color(DIM, "signal"),
        color(DIM, "security"),
        color(DIM, "network")
    );
    for (index, point) in points.iter().enumerate() {
        let security = if point.secured() {
            point.security.clone()
        } else {
            "open".to_string()
        };
        let security = if point.secured() {
            security
        } else {
            color(YELLOW, &security)
        };
        let name = if point.in_use {
            format!("{} {}", point.ssid, color(GREEN, "*"))
        } else {
            point.ssid.clone()
        };
        println!(
            "  {:>2}  {}  {:<9} {}",
            index + 1,
            color(CYAN, &bar(point.signal)),
            security,
            name
        );
    }
}

fn bar(signal: u8) -> String {
    let filled = ((signal.min(100) as usize) + 9) / 10;
    let mut out = String::new();
    for index in 0..10 {
        out.push(if index < filled { '█' } else { '░' });
    }
    out
}

fn join(device: &str, ssid: &str, password: Option<&str>, hidden: bool) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["device", "wifi", "connect", ssid];

    if let Some(password) = password {
        if !password.is_empty() {
            args.push("password");
            args.push(password);
        }
    }
    if hidden {
        args.push("hidden");
        args.push("yes");
    }
    args.push("ifname");
    args.push(device);

    nmcli(&args).map(|_| ())
}

fn print_status(device: &str) {
    let text = match nmcli(&[
        "-t",
        "-f",
        "GENERAL.STATE,GENERAL.CONNECTION,IP4.ADDRESS,IP4.GATEWAY,IP4.DNS",
        "device",
        "show",
        device,
    ]) {
        Ok(text) => text,
        Err(error) => {
            println!("{} {error}", color(RED, "error:"));
            return;
        }
    };

    println!();
    for line in text.lines() {
        let fields = split_terse(line);
        if fields.len() < 2 || fields[1].is_empty() {
            continue;
        }
        let key = fields[0].trim_start_matches("IP4.").to_lowercase();
        println!("  {:<11} {}", color(DIM, &key), fields[1..].join(" "));
    }
}

fn print_devices() {
    if let Ok(text) = nmcli(&["-t", "-f", "DEVICE,TYPE,STATE", "device", "status"]) {
        println!();
        println!("  {}", color(DIM, "devices:"));
        for line in text.lines() {
            let fields = split_terse(line);
            if fields.len() >= 3 {
                println!("    {:<16} {:<10} {}", fields[0], fields[1], fields[2]);
            }
        }
    }
}

/// NMcli terse mode escapes `:` and `\` with a backslash.
fn split_terse(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars();

    while let Some(character) = chars.next() {
        match character {
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            ':' => fields.push(std::mem::take(&mut current)),
            _ => current.push(character),
        }
    }
    fields.push(current);
    fields
}

fn ask_secret(prompt: &str) -> String {
    print!("  {}: ", color(BOLD, prompt));
    io::stdout().flush().ok();

    let _ = Command::new("stty").arg("-echo").status();
    let mut line = String::new();
    let read = io::stdin().read_line(&mut line);
    let _ = Command::new("stty").arg("echo").status();
    println!();

    if read.is_err() {
        return String::new();
    }
    line.trim_end_matches(['\r', '\n']).to_string()
}

fn ask_optional_secret(prompt: &str) -> Option<String> {
    let value = ask_secret(prompt);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn confirm(prompt: &str, default: bool) -> bool {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        print!("  {} {} ", color(BOLD, &format!("{prompt}?")), color(DIM, &format!("[{hint}]")));
        io::stdout().flush().ok();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return default;
        }
        match line.trim().to_lowercase().as_str() {
            "" => return default,
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("{}", color(RED, "  please answer yes or no")),
        }
    }
}

fn fail(message: &str) -> ! {
    eprintln!("{} {message}", color(RED, "error:"));
    std::process::exit(1);
}
