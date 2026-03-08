mod commands;
mod core;
mod mcu;
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
            match core::config::load() {
                Ok(config) => {
                    println!("⚙️  embtool configuration:");
                    println!("   Toolchain default: {:?}", config.toolchain.default);
                    println!("   Registry URL: {}", config.registry.url);
                    println!("   Mirror enabled: {}", config.mirror.enabled);
                    if config.mirror.enabled {
                        println!("   Mirror URL: {}", config.mirror.url);
                        println!("   Mirror fallback: {}", config.mirror.fallback);
                    }
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
