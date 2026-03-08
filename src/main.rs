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

    /// Initialize embtool in current directory (interactive)
    Init,

    /// Create a new embedded project
    New {
        /// Project name
        name: String,
        /// Target MCU (e.g., k10d, k12, k22f, k64, k66)
        #[arg(long)]
        mcu: String,
        /// Project type: application, bootloader, library
        #[arg(long, default_value = "application")]
        r#type: String,
        /// Toolchain spec (e.g., nxp:14.2). Defaults to config default
        #[arg(long)]
        toolchain: Option<String>,
    },

    /// Manage CMake versions
    Cmake {
        #[command(subcommand)]
        action: CmakeAction,
    },

    /// Manage development tools (clang-format, clang-tidy)
    Tool {
        #[command(subcommand)]
        action: ToolAction,
    },

    /// Build the current project
    Build {
        /// Build profile: debug or release
        #[arg(long, default_value = "debug")]
        profile: String,
        /// Clean build directory first
        #[arg(long)]
        clean: bool,
        /// Show verbose CMake output
        #[arg(long, short)]
        verbose: bool,
    },

    /// Format source files with clang-format
    Format {
        /// Check mode (exit 1 if files need formatting)
        #[arg(long)]
        check: bool,
    },

    /// Lint source files with clang-tidy
    Lint {
        /// Auto-fix issues
        #[arg(long)]
        fix: bool,
    },

    /// Show embtool configuration
    Config,
}

#[derive(Subcommand)]
enum CmakeAction {
    /// Install a CMake version (e.g., 3.28, latest)
    Install {
        /// Version spec (e.g., 3.28, 3.28.6, latest)
        version: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// List installed CMake versions
    List {
        #[arg(long)]
        available: bool,
    },
    /// Remove a CMake version
    Remove {
        version: String,
    },
}

#[derive(Subcommand)]
enum ToolAction {
    /// Install a tool (e.g., clang-format 18, clang-tidy 22)
    Install {
        /// Tool name (clang-format, clang-tidy)
        name: String,
        /// Version (e.g., 18, 22)
        version: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// List installed tools
    List {
        /// Filter by tool name
        name: Option<String>,
        #[arg(long)]
        available: bool,
    },
    /// Remove a tool version
    Remove {
        /// Tool name
        name: String,
        /// Version to remove
        version: String,
    },
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

        Commands::Cmake { action } => match action {
            CmakeAction::Install { version, force } => {
                commands::cmake::install(version.as_deref(), force)
            }
            CmakeAction::List { available } => commands::cmake::list(available),
            CmakeAction::Remove { version } => commands::cmake::remove(&version),
        },

        Commands::Tool { action } => match action {
            ToolAction::Install { name, version, force } => {
                commands::tool::install(&name, version.as_deref(), force)
            }
            ToolAction::List { name, available } => {
                if available {
                    let tool_name = name.as_deref().unwrap_or("clang-format");
                    commands::tool::list_available(tool_name)
                } else {
                    commands::tool::list(name.as_deref())
                }
            }
            ToolAction::Remove { name, version } => {
                commands::tool::remove(&name, &version)
            }
        },

        Commands::Init => commands::init::run(),

        Commands::New { name, mcu, r#type, toolchain } => {
            commands::new::run(&name, &mcu, &r#type, toolchain.as_deref())
        }

        Commands::Build { profile, clean, verbose } => {
            commands::build::run(&profile, clean, verbose)
        }

        Commands::Format { check } => commands::format::run(check),

        Commands::Lint { fix } => commands::lint::run(fix),

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
