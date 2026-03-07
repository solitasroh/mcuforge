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
    /// Install a toolchain version
    Install {
        /// Version to install (e.g., 13.3, 12.2)
        version: String,
    },
    /// List installed toolchains
    List,
    /// Set the active toolchain
    Use {
        /// Version to activate
        version: String,
    },
    /// Remove an installed toolchain
    Remove {
        /// Version to remove
        version: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Toolchain { action } => match action {
            ToolchainAction::Install { version } => {
                println!("📦 Installing ARM GCC toolchain v{}...", version);
                println!("   (Not yet implemented)");
            }
            ToolchainAction::List => {
                println!("📋 Installed toolchains:");
                println!("   (Not yet implemented)");
            }
            ToolchainAction::Use { version } => {
                println!("🔄 Switching to ARM GCC v{}...", version);
                println!("   (Not yet implemented)");
            }
            ToolchainAction::Remove { version } => {
                println!("🗑️  Removing ARM GCC v{}...", version);
                println!("   (Not yet implemented)");
            }
        },
        Commands::New { name, mcu } => {
            println!("🆕 Creating project '{}' for MCU '{}'...", name, mcu);
            println!("   (Not yet implemented)");
        }
        Commands::Build => {
            println!("🔨 Building project...");
            println!("   (Not yet implemented)");
        }
        Commands::Config => {
            println!("⚙️  embtool configuration:");
            println!("   (Not yet implemented)");
        }
    }
}
