//! Info command - display project information.

use colored::Colorize;
use std::fs;

use crate::error::CliResult;
use crate::generators::find_project_root;

/// Display project information.
pub async fn run() -> CliResult<()> {
    let project_root = match find_project_root() {
        Ok(root) => root,
        Err(_) => {
            println!();
            println!(
                "  {} Not in an Armature project directory",
                "⚠".yellow()
            );
            println!();
            print_cli_info();
            return Ok(());
        }
    };

    println!();
    println!(
        "{}",
        "  ╔═══════════════════════════════════════════════════════════╗"
            .bright_cyan()
    );
    println!(
        "{}",
        "  ║                   Project Information                     ║"
            .bright_cyan()
    );
    println!(
        "{}",
        "  ╚═══════════════════════════════════════════════════════════╝"
            .bright_cyan()
    );
    println!();

    // Read Cargo.toml
    let cargo_toml_path = project_root.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path)?;

    // Parse basic info
    let name = parse_field(&cargo_toml, "name").unwrap_or_else(|| "Unknown".to_string());
    let version = parse_field(&cargo_toml, "version").unwrap_or_else(|| "0.0.0".to_string());
    let edition = parse_field(&cargo_toml, "edition").unwrap_or_else(|| "2021".to_string());

    println!("  {} {}", "Project:".bright_white().bold(), name.cyan());
    println!("  {} {}", "Version:".bright_white().bold(), version);
    println!("  {} {}", "Edition:".bright_white().bold(), edition);
    println!(
        "  {} {}",
        "Root:".bright_white().bold(),
        project_root.display()
    );
    println!();

    // Count source files
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        let (rust_files, lines) = count_source_files(&src_dir);
        println!(
            "  {} {} Rust files, ~{} lines",
            "Source:".bright_white().bold(),
            rust_files,
            lines
        );
    }

    // Check for common directories
    println!();
    println!("  {}", "Structure:".bright_white().bold());

    let dirs_to_check = [
        ("src/controllers", "Controllers"),
        ("src/services", "Services"),
        ("src/middleware", "Middleware"),
        ("src/guards", "Guards"),
        ("src/models", "Models"),
        ("tests", "Tests"),
    ];

    for (dir, name) in dirs_to_check {
        let path = project_root.join(dir);
        if path.exists() {
            let count = count_rs_files(&path);
            println!(
                "    {} {} ({})",
                "✓".green(),
                name,
                format!("{} files", count).dimmed()
            );
        }
    }

    // Check for Docker
    if project_root.join("Dockerfile").exists() {
        println!("    {} Docker", "✓".green());
    }

    // Check for environment files
    if project_root.join(".env").exists() {
        println!("    {} .env configured", "✓".green());
    } else if project_root.join(".env.example").exists() {
        println!(
            "    {} .env.example present (copy to .env)",
            "○".yellow()
        );
    }

    println!();
    print_cli_info();

    Ok(())
}

/// Print CLI version and help information.
fn print_cli_info() {
    println!("  {}", "Armature CLI:".bright_white().bold());
    println!(
        "    {} armature {}",
        "Version:".dimmed(),
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("  {}", "Quick Commands:".bright_white().bold());
    println!("    {} armature dev", "→".cyan());
    println!("    {} armature generate controller <name>", "→".cyan());
    println!("    {} armature build --release", "→".cyan());
    println!();
}

/// Parse a field from Cargo.toml content.
fn parse_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(field) && line.contains('=') {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                return Some(
                    parts[1]
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
        }
    }
    None
}

/// Count Rust source files in a directory.
fn count_rs_files(dir: &std::path::Path) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "rs")
                .unwrap_or(false)
        })
        .count()
}

/// Count source files and approximate lines.
fn count_source_files(dir: &std::path::Path) -> (usize, usize) {
    let mut files = 0;
    let mut lines = 0;

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry
            .path()
            .extension()
            .map(|ext| ext == "rs")
            .unwrap_or(false)
        {
            files += 1;
            if let Ok(content) = fs::read_to_string(entry.path()) {
                lines += content.lines().count();
            }
        }
    }

    (files, lines)
}

