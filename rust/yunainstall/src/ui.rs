//! Terminal UI helpers: colors, prompts, menus, secret input.

use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static COLOR: AtomicBool = AtomicBool::new(false);

pub fn init_color() {
    COLOR.store(io::stdout().is_terminal(), Ordering::Relaxed);
}

fn on() -> bool {
    COLOR.load(Ordering::Relaxed)
}

pub fn paint(code: &str, text: &str) -> String {
    if on() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}
pub fn cyan(text: &str) -> String {
    paint("36", text)
}
pub fn dim(text: &str) -> String {
    paint("2", text)
}
pub fn green(text: &str) -> String {
    paint("32", text)
}
pub fn yellow(text: &str) -> String {
    paint("33", text)
}
pub fn red(text: &str) -> String {
    paint("31", text)
}

pub fn banner() {
    let art = r#"
   __  __ _   _ _  _    _
   \ \/ /| | | | \| |  /_\
    \  / | |_| | .` | / _ \
    /  \ |  _  | |\  |/ ___ \
   /_/\_\|_| |_|_| \_/_/   \_\
"#;
    println!("{}", cyan(art));
    println!("        {} {}", bold("Yuna OS"), dim("installer"));
    println!();
}

pub fn heading(text: &str) {
    println!();
    println!("{}", cyan(&format!("── {text} ")));
}

pub fn ask(prompt: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{} ", bold(&format!("{prompt}:")));
    } else {
        print!("{} {} ", bold(&format!("{prompt}:")), dim(&format!("[{default}]")));
    }
    io::stdout().flush().ok();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return default.to_string();
    }
    let line = line.trim();
    if line.is_empty() {
        default.to_string()
    } else {
        line.to_string()
    }
}

pub fn confirm(prompt: &str, default: bool) -> bool {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        print!("{} {} ", bold(&format!("{prompt}?")), dim(&format!("[{hint}]")));
        io::stdout().flush().ok();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return default;
        }
        match line.trim().to_lowercase().as_str() {
            "" => return default,
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("{}", red("please answer yes or no")),
        }
    }
}

/// Numbered menu. Returns the zero-based index of the chosen entry.
pub fn select(prompt: &str, options: &[String], default: usize) -> usize {
    println!("{}", bold(&format!("{prompt}:")));
    for (index, option) in options.iter().enumerate() {
        let marker = if index == default {
            cyan("▶")
        } else {
            dim(" ")
        };
        println!("  {marker} {}) {}", index + 1, option);
    }

    loop {
        print!("{} ", dim(&format!("[1-{}] ({})", options.len(), default + 1)));
        io::stdout().flush().ok();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return default;
        }
        let line = line.trim();
        if line.is_empty() {
            return default;
        }
        if let Ok(choice) = line.parse::<usize>() {
            if choice >= 1 && choice <= options.len() {
                return choice - 1;
            }
        }
        println!("{}", red("invalid choice"));
    }
}


/// Reads a line without echoing it.
pub fn ask_secret(prompt: &str) -> String {
    print!("{} ", bold(&format!("{prompt}:")));
    io::stdout().flush().ok();

    // `stty` works when stdin is a terminal; otherwise it fails harmlessly.
    let _ = Command::new("stty").arg("-echo").status();

    let mut line = String::new();
    let result = io::stdin().read_line(&mut line);

    let _ = Command::new("stty").arg("echo").status();
    println!();

    if result.is_err() {
        return String::new();
    }
    line.trim_end_matches(['\n', '\r']).to_string()
}
