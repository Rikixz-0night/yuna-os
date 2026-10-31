//! yuna-fetch - system information for Yuna OS.
//!
//! Dependency-free: reads /proc, /sys and a couple of optional commands.
//!
//!   yuna-fetch            logo + info
//!   yuna-fetch --no-logo  info only
//!   yuna-fetch --json     machine readable
//!   yuna-fetch --short    compact single line

use std::env;
use std::fs;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::Command;

const LOGO: &[&str] = &[
    r"    __  __ _   _ _  _    _    ",
    r"    \ \/ /| | | | \| |  /_\   ",
    r"     \  / | |_| | .` | / _ \  ",
    r"     /  \ |  _  | |\  |/ ___ \ ",
    r"    /_/\_\|_| |_|_| \_/_/   \_\",
];

const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

struct Field {
    key: &'static str,
    value: String,
}

impl Field {
    fn new(key: &'static str, value: impl Into<String>) -> Self {
        Self { key, value: value.into() }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut show_logo = true;
    let mut json = false;
    let mut short = false;

    for arg in &args {
        match arg.as_str() {
            "--no-logo" | "-l" => show_logo = false,
            "--json" | "-j" => json = true,
            "--short" | "-s" => short = true,
            "--help" | "-h" => {
                println!("yuna-fetch {}", env!("CARGO_PKG_VERSION"));
                println!("usage: yuna-fetch [--no-logo] [--json] [--short]");
                return;
            }
            "--version" | "-v" => {
                println!("yuna-fetch {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            other => {
                eprintln!("yuna-fetch: unknown option `{other}` (try --help)");
                std::process::exit(2);
            }
        }
    }

    let fields = collect();
    let color = io::stdout().is_terminal();

    if json {
        print_json(&fields);
    } else if short {
        let summary: Vec<String> = fields.iter().map(|f| f.value.clone()).collect();
        println!("{}", summary.join(" | "));
    } else {
        print_pretty(&fields, show_logo, color);
    }
}

fn collect() -> Vec<Field> {
    let mut fields = Vec::new();

    fields.push(Field::new("os", os_pretty_name()));
    if let Some(host) = host_model() {
        fields.push(Field::new("host", host));
    }
    fields.push(Field::new("kernel", kernel_release()));
    fields.push(Field::new("uptime", uptime()));
    fields.push(Field::new("packages", package_count()));
    fields.push(Field::new("shell", shell()));
    if let Some(de) = desktop() {
        fields.push(Field::new("de", de));
    }
    if let Some(wm) = window_manager() {
        fields.push(Field::new("wm", wm));
    }
    fields.push(Field::new("terminal", terminal()));
    fields.push(Field::new("cpu", cpu_model()));
    if let Some(gpu) = gpu() {
        fields.push(Field::new("gpu", gpu));
    }
    fields.push(Field::new("memory", memory()));
    if let Some(disk) = disk_usage() {
        fields.push(Field::new("disk", disk));
    }
    if let Some(ip) = local_ip() {
        fields.push(Field::new("local ip", ip));
    }

    fields
}

fn print_pretty(fields: &[Field], show_logo: bool, color: bool) {
    let (cyan, bold, dim, reset) = if color {
        (CYAN, BOLD, DIM, RESET)
    } else {
        ("", "", "", "")
    };

    let logo_width = if show_logo {
        LOGO.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 4
    } else {
        0
    };

    let rows = fields.len().max(if show_logo { LOGO.len() } else { 0 });

    for row in 0..rows {
        let mut line = String::new();

        if show_logo {
            let art = LOGO.get(row).copied().unwrap_or("");
            line.push_str(cyan);
            line.push_str(art);
            line.push_str(reset);
            let pad = logo_width.saturating_sub(art.chars().count());
            line.push_str(&" ".repeat(pad));
        }

        if let Some(field) = fields.get(row) {
            if row == 0 {
                line.push_str(bold);
                line.push_str(cyan);
                line.push_str(&field.value);
                line.push_str(reset);
            } else {
                line.push_str(cyan);
                line.push_str(field.key);
                line.push_str(reset);
                line.push_str(dim);
                line.push_str(" ~ ");
                line.push_str(reset);
                line.push_str(&field.value);
            }
        }

        println!("{}", line.trim_end());

        if row == 0 {
            let width = fields.iter().map(|f| f.value.len()).max().unwrap_or(0).max(24);
            let rule = "─".repeat(width.min(60));
            if show_logo {
                print!("{}{}{}{}", " ".repeat(logo_width), dim, rule, reset);
                println!();
            } else {
                println!("{dim}{rule}{reset}");
            }
        }
    }

    if color {
        println!();
        print!("{}", " ".repeat(logo_width));
        for code in 40..48 {
            print!("\x1b[{code}m   ");
        }
        print!("{reset}");
        println!();
    }
}

fn print_json(fields: &[Field]) {
    println!("{{");
    for (index, field) in fields.iter().enumerate() {
        let comma = if index + 1 == fields.len() { "" } else { "," };
        println!("  \"{}\": \"{}\"{}", json_escape(field.key), json_escape(&field.value), comma);
    }
    println!("}}");
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/* ------------------------------- probes ------------------------------- */

fn os_pretty_name() -> String {
    if let Ok(text) = fs::read_to_string("/etc/os-release") {
        if let Some(value) = key_value(&text, "PRETTY_NAME") {
            return value;
        }
    }
    "Linux".to_string()
}

fn key_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{key}=")) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn read_trim(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn host_model() -> Option<String> {
    let vendor = read_trim("/sys/devices/virtual/dmi/id/sys_vendor").unwrap_or_default();
    let product = read_trim("/sys/devices/virtual/dmi/id/product_name")?;
    if vendor.is_empty() || vendor == "System manufacturer" {
        Some(product)
    } else {
        Some(format!("{vendor} {product}"))
    }
}

fn kernel_release() -> String {
    read_trim("/proc/sys/kernel/osrelease").unwrap_or_else(|| "unknown".into())
}

fn uptime() -> String {
    let Ok(text) = fs::read_to_string("/proc/uptime") else {
        return "unknown".into();
    };
    let seconds = text
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;

    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn package_count() -> String {
    let dir = "/var/lib/pacman/local";
    match fs::read_dir(dir) {
        Ok(entries) => {
            let count = entries
                .flatten()
                .filter(|e| e.file_name() != "ALPM_DB_VERSION")
                .count();
            format!("{count} (pacman)")
        }
        Err(_) => "unknown".into(),
    }
}

fn shell() -> String {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let name = Path::new(&shell)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("sh")
        .to_string();
    let version = Command::new(&shell)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()));

    match version {
        Some(line) if !line.is_empty() => {
            // "GNU bash, version 5.2.37(1)-release ..." -> "bash 5.2.37"
            let version = line
                .split_whitespace()
                .find(|w| w.chars().next().is_some_and(|c| c.is_ascii_digit()))
                .unwrap_or("")
                .split(['(', '-'])
                .next()
                .unwrap_or("")
                .to_string();
            if version.is_empty() {
                name
            } else {
                format!("{name} {version}")
            }
        }
        _ => name,
    }
}

fn desktop() -> Option<String> {
    for var in ["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION", "XDG_SESSION_DESKTOP"] {
        if let Ok(value) = env::var(var) {
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn window_manager() -> Option<String> {
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return Some("Hyprland".into());
    }
    if env::var_os("SWAYSOCK").is_some() {
        return Some("Sway".into());
    }
    None
}

fn terminal() -> String {
    env::var("TERM").unwrap_or_else(|_| "unknown".into())
}

fn cpu_model() -> String {
    let Ok(text) = fs::read_to_string("/proc/cpuinfo") else {
        return "unknown".into();
    };
    for line in text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "model name" {
                return value.split_whitespace().collect::<Vec<_>>().join(" ");
            }
        }
    }
    "unknown".into()
}

fn gpu() -> Option<String> {
    let output = Command::new("lspci").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("vga compatible") || lower.contains("3d controller") || lower.contains("display controller") {
            if let Some((_, name)) = line.split_once(": ") {
                return Some(name.trim().to_string());
            }
        }
    }
    None
}

fn memory() -> String {
    let Ok(text) = fs::read_to_string("/proc/meminfo") else {
        return "unknown".into();
    };
    let mut total = 0u64;
    let mut available = 0u64;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total = parse_kib(value);
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available = parse_kib(value);
        }
    }
    if total == 0 {
        return "unknown".into();
    }
    let used = total.saturating_sub(available);
    let pct = used as f64 / total as f64 * 100.0;
    format!("{} / {} ({:.0}%)", human_kib(used), human_kib(total), pct)
}

fn parse_kib(value: &str) -> u64 {
    value
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn human_kib(kib: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    let mut value = kib as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn disk_usage() -> Option<String> {
    let output = Command::new("df").args(["-h", "--output=used,size,pcent", "/"]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().nth(1)?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 {
        Some(format!("{} / {} ({})", parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

fn local_ip() -> Option<String> {
    let output = Command::new("ip")
        .args(["-4", "-o", "route", "get", "1.1.1.1"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut words = text.split_whitespace();
    while let Some(word) = words.next() {
        if word == "src" {
            return words.next().map(|s| s.to_string());
        }
    }
    None
}
