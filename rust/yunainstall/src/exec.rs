//! Command execution with a dry-run mode.

use std::process::Command;

use crate::ui;

pub struct Runner {
    pub dry_run: bool,
}

impl Runner {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    fn log(&self, program: &str, args: &[String]) {
        let rendered = if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args.join(" "))
        };
        println!("  {} {}", ui::dim("+"), rendered);
    }

    pub fn run(&self, program: &str, args: &[String]) -> Result<(), String> {
        self.log(program, args);
        if self.dry_run {
            return Ok(());
        }

        let status = Command::new(program)
            .args(args)
            .status()
            .map_err(|error| format!("failed to start `{program}`: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("`{program}` exited with {status}"))
        }
    }

    pub fn shell(&self, script: &str) -> Result<(), String> {
        println!("  {} {}", ui::dim("+"), ui::dim("sh -c <inline script>"));
        if self.dry_run {
            return Ok(());
        }

        let status = Command::new("sh")
            .arg("-c")
            .arg(script)
            .status()
            .map_err(|error| format!("failed to start sh: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("shell command failed with {status}"))
        }
    }

    /// Runs a command and returns trimmed stdout. In dry-run mode returns a
    /// deterministic placeholder so later steps can still be rendered.
    pub fn capture(&self, program: &str, args: &[String]) -> Result<String, String> {
        self.log(program, args);
        if self.dry_run {
            return Ok("DRYRUN-UUID".to_string());
        }

        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|error| format!("failed to start `{program}`: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "`{program}` failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn write(&self, path: &str, contents: &str) -> Result<(), String> {
        println!("  {} write {}", ui::dim("+"), path);
        if self.dry_run {
            for line in contents.lines().take(400) {
                println!("      {}", ui::dim(line));
            }
            return Ok(());
        }

        std::fs::write(path, contents).map_err(|error| format!("cannot write {path}: {error}"))
    }
}
