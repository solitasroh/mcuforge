use anyhow::Result;
use iocraft::prelude::*;

use crate::core::{builder::{self, BuildProfile}, project};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant};

pub fn run(profile: &str, clean: bool, verbose: bool) -> Result<()> {
    // Find and load project
    let project_path = project::find_project()?;
    let project_dir = project_path.parent().unwrap();
    let proj = project::load(&project_path)?;
    let build_profile = BuildProfile::from_str(profile)?;

    // Header
    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(title: format!("embtool build ({})", build_profile.label()))
            Section(title: proj.project.name.clone()) {
                Entry(label: "MCU".to_string(), value: format!("{} ({})", proj.target.mcu, proj.target.core))
                Entry(label: "Toolchain".to_string(), value: proj.toolchain.id())
            }
        }
    });
    println!();

    // Configure + Build
    ui::render(element! {
        StatusLine(
            icon: "⚙".to_string(),
            message: "Configuring...".to_string(),
            variant: StatusVariant::Info,
        )
    });

    match builder::build(project_dir, &proj, build_profile, verbose, clean) {
        Ok(result) => {
            println!();

            // Build progress bar
            let flash_bar = progress_bar(result.flash_pct(), 20);
            let ram_bar = progress_bar(result.ram_pct(), 20);

            ui::render(element! {
                Section(title: "Build Successful".to_string(), variant: SectionVariant::Success) {
                    Entry(label: "Time".to_string(), value: format!("{:.1}s", result.build_time_secs))
                    Entry(label: "Output".to_string(), value: result.elf_path.display().to_string())
                }
            });

            if let Some(ref bin) = result.bin_path {
                ui::render(element! {
                    StatusLine(
                        icon: "·".to_string(),
                        message: format!("  .bin  {}", bin.display()),
                        variant: StatusVariant::Muted,
                    )
                });
            }
            if let Some(ref hex) = result.hex_path {
                ui::render(element! {
                    StatusLine(
                        icon: "·".to_string(),
                        message: format!("  .hex  {}", hex.display()),
                        variant: StatusVariant::Muted,
                    )
                });
            }

            // Memory usage report
            if result.flash_total > 0 {
                println!();
                ui::render(element! {
                    Section(title: "Memory Usage".to_string()) {
                        Entry(
                            label: "Flash".to_string(),
                            value: format!(
                                "{:>7} / {:>7} ({:>5.1}%)  {}",
                                format_bytes(result.flash_used),
                                format_bytes(result.flash_total),
                                result.flash_pct(),
                                flash_bar,
                            ),
                        )
                        Entry(
                            label: "RAM".to_string(),
                            value: format!(
                                "{:>7} / {:>7} ({:>5.1}%)  {}",
                                format_bytes(result.ram_used),
                                format_bytes(result.ram_total),
                                result.ram_pct(),
                                ram_bar,
                            ),
                        )
                    }
                });
            }

            Ok(())
        }
        Err(e) => {
            println!();
            ui::render(element! {
                Section(title: "Build Failed".to_string()) {
                    StatusLine(
                        icon: "✗".to_string(),
                        message: format!("{}", e),
                        variant: StatusVariant::Error,
                    )
                }
            });
            Err(e)
        }
    }
}

fn progress_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}",
        "█".repeat(filled),
        "░".repeat(empty),
    )
}

fn format_bytes(bytes: u32) -> String {
    if bytes >= 1024 * 1024 {
        format!("{}MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{}B", bytes)
    }
}
