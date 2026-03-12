use anyhow::Result;
use clap::Subcommand;
use iocraft::prelude::*;

use crate::core::{claude, project};
use crate::ui::{self, Header, Section, SectionVariant, StatusLine, StatusVariant};

#[derive(Subcommand)]
pub enum ClaudeAction {
    /// Install Claude Code skills to current project
    Install {
        /// Install specific skill only
        skill: Option<String>,
        /// Force reinstall (overwrite existing)
        #[arg(short, long)]
        force: bool,
    },
    /// Update skills to latest version
    Update {
        /// Update specific skill only
        skill: Option<String>,
    },
    /// List installed skills
    List {
        /// Show all available skills
        #[arg(short, long)]
        all: bool,
    },
    /// Sync skills with embtool.toml configuration
    Sync,
    /// Show skill status
    Status,
}

pub fn run(action: ClaudeAction) -> Result<()> {
    match action {
        ClaudeAction::Install { skill: _, force } => cmd_install(force),
        ClaudeAction::Update { skill: _ } => cmd_update(),
        ClaudeAction::List { all } => cmd_list(all),
        ClaudeAction::Sync => cmd_sync(),
        ClaudeAction::Status => cmd_status(),
    }
}

fn cmd_install(force: bool) -> Result<()> {
    let (project_dir, config) = load_project()?;

    ui::render(element! {
        Header(
            title: "mcuforge claude install".to_string(),
        )
    });
    println!();

    let cache_dir = claude::download_skills_package(
        config
            .claude
            .as_ref()
            .and_then(|c| c.version.as_deref()),
    )?;

    let report = claude::install_skills(&project_dir, &config, &cache_dir, force)?;

    ui::render(element! {
        Section(title: "Installed".to_string(), variant: SectionVariant::Success) {
            #(report.installed.iter().map(|s| {
                element! {
                    Text(content: format!("   ✓ {}", s), color: Some(Color::Green))
                }
            }))
            #(report.templates.iter().map(|s| {
                element! {
                    Text(content: format!("   ✓ {} (template)", s), color: Some(Color::Cyan))
                }
            }))
            #(report.skipped.iter().map(|s| {
                element! {
                    Text(content: format!("   ⊘ {} (skipped)", s), color: Some(Color::DarkGrey))
                }
            }))
        }
    });

    println!();
    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!("{} skills installed to .claude/skills/", report.total()),
            variant: StatusVariant::Success,
        )
    });

    Ok(())
}

fn cmd_update() -> Result<()> {
    ui::render(element! {
        Header(title: "mcuforge claude update".to_string())
    });
    println!();

    println!("  Checking for updates...");
    // TODO: Compare installed version with latest GitHub release
    println!("  Update not yet implemented. Use 'mcuforge claude install --force' for now.");
    Ok(())
}

fn cmd_list(_all: bool) -> Result<()> {
    let (project_dir, _config) = load_project()?;
    let skills_dir = project_dir.join(".claude").join("skills");

    ui::render(element! {
        Header(title: "mcuforge claude list".to_string())
    });
    println!();

    if !skills_dir.exists() {
        println!("  No Claude skills installed. Run 'mcuforge claude install' first.");
        return Ok(());
    }

    println!("  {:>3} │ {:<28} │ Status", "#", "Skill");
    println!("  ────┼──────────────────────────────┼─────────");

    let mut count = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&skills_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        count += 1;
        println!("  {:>3} │ {:<28} │ installed", count, name);
    }

    println!();
    ui::render(element! {
        StatusLine(
            icon: "i".to_string(),
            message: format!("Total: {} skills", count),
            variant: StatusVariant::Info,
        )
    });

    Ok(())
}

fn cmd_sync() -> Result<()> {
    let (project_dir, config) = load_project()?;

    ui::render(element! {
        Header(title: "mcuforge claude sync".to_string())
    });
    println!();

    let cache_dir = claude::download_skills_package(
        config
            .claude
            .as_ref()
            .and_then(|c| c.version.as_deref()),
    )?;

    let report = claude::install_skills(&project_dir, &config, &cache_dir, true)?;

    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!(
                "Synced: {} installed, {} templates, {} skipped",
                report.installed.len(),
                report.templates.len(),
                report.skipped.len()
            ),
            variant: StatusVariant::Success,
        )
    });

    Ok(())
}

fn cmd_status() -> Result<()> {
    let (project_dir, config) = load_project()?;
    let skills_dir = project_dir.join(".claude").join("skills");

    ui::render(element! {
        Header(title: "mcuforge claude status".to_string())
    });
    println!();

    let installed_count = if skills_dir.exists() {
        std::fs::read_dir(&skills_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    && !e.file_name().to_string_lossy().starts_with('.')
            })
            .count()
    } else {
        0
    };

    ui::render(element! {
        Section(title: "Status".to_string()) {
            ui::Entry(label: "Project".to_string(), value: config.project.name.clone())
            ui::Entry(label: "MCU".to_string(), value: config.target.mcu.clone())
            ui::Entry(label: "Skills installed".to_string(), value: installed_count.to_string())
        }
    });

    if let Some(ref claude_cfg) = config.claude {
        println!();
        ui::render(element! {
            Section(title: "Claude Config".to_string()) {
                ui::Entry(
                    label: "Version".to_string(),
                    value: claude_cfg.version.clone().unwrap_or_else(|| "(none)".into()),
                )
                ui::Entry(
                    label: "GitLab".to_string(),
                    value: claude_cfg.gitlab.as_ref()
                        .map(|g| g.url.clone())
                        .unwrap_or_else(|| "(not configured)".into()),
                )
                ui::Entry(
                    label: "OpenProject".to_string(),
                    value: claude_cfg.openproject.as_ref()
                        .map(|o| format!("{} (project #{})", o.url, o.project_id))
                        .unwrap_or_else(|| "(not configured)".into()),
                )
            }
        });
    } else {
        println!();
        println!("  [claude] section not found in embtool.toml");
        println!("  Add it with: mcuforge init --claude");
    }

    Ok(())
}

fn load_project() -> Result<(std::path::PathBuf, project::ProjectConfig)> {
    let project_path = project::find_project()?;
    let config = project::load(&project_path)?;
    let project_dir = project_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    Ok((project_dir, config))
}
