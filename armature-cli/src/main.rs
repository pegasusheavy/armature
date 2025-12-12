//! Armature CLI - Code generation and development tools for Armature framework.
//!
//! # Commands
//!
//! - `armature new <name>` - Create a new Armature project
//! - `armature generate <type> <name>` - Generate code (controller, module, etc.)
//! - `armature dev` - Run development server with file watching
//! - `armature build` - Build for production
//! - `armature info` - Show project information

use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod error;
mod generators;
mod templates;
mod watcher;

use commands::{build, dev, generate, info, new};
use error::CliResult;

/// Armature CLI - Modern Rust Web Framework Tools
#[derive(Parser)]
#[command(name = "armature")]
#[command(author = "Pegasus Heavy Industries LLC")]
#[command(version)]
#[command(about = "CLI tool for Armature framework - code generation and development server")]
#[command(long_about = r#"
Armature CLI provides tools for:
  • Creating new projects from templates
  • Generating controllers, modules, middleware, guards, and services
  • Running a development server with hot reloading
  • Building for production

Examples:
  armature new my-api              Create a new project
  armature generate controller users   Generate a UsersController
  armature dev                     Start development server
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Armature project
    #[command(alias = "n")]
    New {
        /// Project name
        name: String,

        /// Template to use (minimal, full, microservice)
        #[arg(short, long, default_value = "minimal")]
        template: String,

        /// Skip git initialization
        #[arg(long)]
        skip_git: bool,

        /// Skip dependency installation
        #[arg(long)]
        skip_install: bool,
    },

    /// Generate code (controller, module, middleware, guard, service)
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        generator: GeneratorType,
    },

    /// Run development server with file watching
    #[command(alias = "d")]
    Dev {
        /// Port to run the server on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Additional cargo arguments
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },

    /// Build the project for production
    #[command(alias = "b")]
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Additional cargo arguments
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },

    /// Show project information
    Info,
}

#[derive(Subcommand)]
enum GeneratorType {
    /// Generate a controller
    #[command(alias = "c")]
    Controller {
        /// Controller name (e.g., "users" or "api/users")
        name: String,

        /// Generate CRUD endpoints
        #[arg(long)]
        crud: bool,

        /// Skip test file generation
        #[arg(long)]
        skip_tests: bool,
    },

    /// Generate a module
    #[command(alias = "m")]
    Module {
        /// Module name
        name: String,

        /// Controllers to include (comma-separated)
        #[arg(short, long)]
        controllers: Option<String>,

        /// Providers/services to include (comma-separated)
        #[arg(short, long)]
        providers: Option<String>,
    },

    /// Generate middleware
    #[command(alias = "mw")]
    Middleware {
        /// Middleware name
        name: String,

        /// Skip test file generation
        #[arg(long)]
        skip_tests: bool,
    },

    /// Generate a guard
    #[command(alias = "gu")]
    Guard {
        /// Guard name
        name: String,

        /// Skip test file generation
        #[arg(long)]
        skip_tests: bool,
    },

    /// Generate a service/provider
    #[command(alias = "s")]
    Service {
        /// Service name
        name: String,

        /// Skip test file generation
        #[arg(long)]
        skip_tests: bool,
    },

    /// Generate a complete resource (controller + service + module)
    #[command(alias = "r")]
    Resource {
        /// Resource name (e.g., "users")
        name: String,

        /// Generate CRUD endpoints
        #[arg(long)]
        crud: bool,
    },
}

fn print_banner() {
    println!(
        "{}",
        r#"
    _                         _
   / \   _ __ _ __ ___   __ _| |_ _   _ _ __ ___
  / _ \ | '__| '_ ` _ \ / _` | __| | | | '__/ _ \
 / ___ \| |  | | | | | | (_| | |_| |_| | | |  __/
/_/   \_\_|  |_| |_| |_|\__,_|\__|\__,_|_|  \___|
"#
        .bright_cyan()
    );
    println!(
        "  {} {}\n",
        "Armature CLI".bright_white().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Print banner for interactive commands
    match &cli.command {
        Commands::New { .. } | Commands::Generate { .. } => print_banner(),
        _ => {}
    }

    let result: CliResult<()> = match cli.command {
        Commands::New {
            name,
            template,
            skip_git,
            skip_install,
        } => new::run(&name, &template, skip_git, skip_install).await,

        Commands::Generate { generator } => match generator {
            GeneratorType::Controller {
                name,
                crud,
                skip_tests,
            } => generate::controller(&name, crud, skip_tests).await,

            GeneratorType::Module {
                name,
                controllers,
                providers,
            } => generate::module(&name, controllers.as_deref(), providers.as_deref()).await,

            GeneratorType::Middleware { name, skip_tests } => {
                generate::middleware(&name, skip_tests).await
            }

            GeneratorType::Guard { name, skip_tests } => generate::guard(&name, skip_tests).await,

            GeneratorType::Service { name, skip_tests } => {
                generate::service(&name, skip_tests).await
            }

            GeneratorType::Resource { name, crud } => generate::resource(&name, crud).await,
        },

        Commands::Dev {
            port,
            host,
            cargo_args,
        } => dev::run(port, &host, &cargo_args).await,

        Commands::Build {
            release,
            cargo_args,
        } => build::run(release, &cargo_args).await,

        Commands::Info => info::run().await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
