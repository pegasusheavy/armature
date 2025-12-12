//! Build command for production builds.

use colored::Colorize;
use std::process::Command;
use std::time::Instant;

use crate::error::{CliError, CliResult};
use crate::generators::find_project_root;

/// Build the project for production.
pub async fn run(release: bool, cargo_args: &[String]) -> CliResult<()> {
    let project_root = find_project_root()?;

    println!();
    println!("  {} Building project...", "→".cyan().bold());
    println!();

    let start = Instant::now();

    let mut args = vec!["build".to_string()];

    if release {
        args.push("--release".to_string());
        println!(
            "  {} Building in {} mode",
            "→".green(),
            "release".green().bold()
        );
    } else {
        println!("  {} Building in {} mode", "→".yellow(), "debug".yellow());
    }

    args.extend(cargo_args.iter().cloned());

    let status = Command::new("cargo")
        .args(&args)
        .current_dir(&project_root)
        .status()?;

    let duration = start.elapsed();

    println!();

    if status.success() {
        println!(
            "  {} Build completed in {:.2}s",
            "✓".green().bold(),
            duration.as_secs_f64()
        );

        if release {
            // Find the binary name from Cargo.toml
            let cargo_toml_path = project_root.join("Cargo.toml");
            let cargo_toml = std::fs::read_to_string(&cargo_toml_path)?;
            let binary_name = parse_package_name(&cargo_toml).unwrap_or("app".to_string());

            let binary_path = project_root.join("target/release").join(&binary_name);
            if binary_path.exists() {
                let metadata = std::fs::metadata(&binary_path)?;
                let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

                println!();
                println!("  {} Binary: {}", "→".dimmed(), binary_path.display());
                println!("  {} Size: {:.2} MB", "→".dimmed(), size_mb);
            }
        }

        Ok(())
    } else {
        Err(CliError::Build("Build failed".to_string()))
    }
}

/// Parse the package name from Cargo.toml content.
fn parse_package_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
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
