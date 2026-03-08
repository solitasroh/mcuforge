mod commands;
mod core;
mod mcu;
mod ui;
mod utils;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "embtool")]
#[command(about = "Embedded toolchain manager for NXP and STM32 MCUs")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up project environment (install toolchain, generate cmake)
    Setup {
        /// CI mode — disable interactive prompts
        #[arg(long)]
        ci: bool,

        /// Force re-setup even if already configured
        #[arg(long)]
        force: bool,
    },

    /// Manage ARM GCC toolchains
    Toolchain {
        #[command(subcommand)]
        action: ToolchainAction,
    },

    /// Create a new embedded project
    New {
        /// Project name
        name: String,
        /// Target MCU (e.g., k22f, k64, k66)
        #[arg(long)]
        mcu: String,
    },

    /// Build the current project
    Build,

    /// Show embtool configuration
    Config,
}

#[derive(Subcommand)]
enum ToolchainAction {
    /// Install a toolchain version (e.g., nxp:14.2, stm:13.3)
    Install {
        /// Vendor:version spec
        spec: String,
        /// Force reinstall
        #[arg(long)]
        force: bool,
    },
    /// List installed toolchains
    List {
        /// Also show available (remote) versions
        #[arg(long)]
        available: bool,
    },
    /// Set the active toolchain
    Use {
        /// Vendor:version to activate
        spec: String,
    },
    /// Remove an installed toolchain
    Remove {
        /// Vendor:version to remove
        spec: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Setup { ci, force } => commands::setup::run(ci, force),

        Commands::Toolchain { action } => match action {
            ToolchainAction::Install { spec, force } => {
                commands::toolchain::install(&spec, force)
            }
            ToolchainAction::List { available } => commands::toolchain::list(available),
            ToolchainAction::Use { spec } => commands::toolchain::use_version(&spec),
            ToolchainAction::Remove { spec } => commands::toolchain::remove(&spec),
        },

        Commands::New { name, mcu } => {
            println!("🆕 Creating project '{}' for MCU '{}'...", name, mcu);
            println!("   (Not yet implemented)");
            Ok(())
        }

        Commands::Build => {
            println!("🔨 Building project...");
            println!("   (Not yet implemented)");
            Ok(())
        }

        Commands::Config => {
            use iocraft::prelude::*;
            match core::config::load() {
                Ok(config) => {
                    ui::render(element! {
                        View(flex_direction: FlexDirection::Column) {
                            ui::Header(
                                title: "embtool config".to_string(),
                            )
                            ui::Section(title: "Toolchain".to_string()) {
                                ui::Entry(
                                    label: "Default".to_string(),
                                    value: config.toolchain.default.unwrap_or_else(|| "(none)".to_string()),
                                )
                            }
                            ui::Section(title: "Registry".to_string()) {
                                ui::Entry(
                                    label: "URL".to_string(),
                                    value: config.registry.url.clone(),
                                )
                                ui::Entry(
                                    label: "Cache TTL".to_string(),
                                    value: format!("{}h", config.registry.cache_ttl_hours),
                                )
                            }
                            ui::Section(title: "Mirror".to_string()) {
                                ui::Entry(
                                    label: "Enabled".to_string(),
                                    value: config.mirror.enabled.to_string(),
                                )
                                ui::Entry(
                                    label: "URL".to_string(),
                                    value: if config.mirror.url.is_empty() { "(none)".to_string() } else { config.mirror.url },
                                )
                                ui::Entry(
                                    label: "Fallback".to_string(),
                                    value: config.mirror.fallback.to_string(),
                                )
                            }
                        }
                    });
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {:#}", e);
        std::process::exit(1);
    }
}
