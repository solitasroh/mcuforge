use anyhow::{bail, Result};
use iocraft::prelude::*;

use crate::core::{config, toolchain_manager, toolchain_registry};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant, ToolchainRow};
use crate::utils::paths;

/// Parse "vendor:version" spec
pub fn parse_spec(spec: &str) -> Result<(String, String)> {
    if let Some((vendor, version)) = spec.split_once(':') {
        Ok((vendor.to_string(), version.to_string()))
    } else {
        bail!(
            "Invalid toolchain spec '{}'. Use 'vendor:version' format (e.g., nxp:14.2, stm:13.3)",
            spec
        );
    }
}

pub fn install(spec: &str, force: bool) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;

    ui::render(element! {
        Header(
            title: "embtool toolchain install".to_string(),
        )
    });

    ui::render(element! {
        StatusLine(
            icon: "↓".to_string(),
            message: format!("Installing {} ARM GCC {}...", vendor.to_uppercase(), version),
            variant: StatusVariant::Info,
        )
    });

    let result = toolchain_manager::install(&vendor, &version, force)?;

    // Check if it was already installed (size_mb == 0 means we skipped)
    if result.size_mb == 0 && !force {
        ui::render(element! {
            Section(title: format!("{}", result.name), variant: SectionVariant::Success) {
                StatusLine(
                    icon: "✓".to_string(),
                    message: "Already installed".to_string(),
                    variant: StatusVariant::Success,
                )
                Entry(label: "GCC".to_string(), value: result.gcc_version)
                Entry(label: "Path".to_string(), value: result.path.display().to_string())
            }
        });
    } else {
        ui::render(element! {
            Section(title: format!("{}", result.name), variant: SectionVariant::Success) {
                StatusLine(
                    icon: "✓".to_string(),
                    message: "Installed successfully".to_string(),
                    variant: StatusVariant::Success,
                )
                Entry(label: "GCC".to_string(), value: result.gcc_version)
                Entry(label: "Size".to_string(), value: format!("{} MB", result.size_mb))
                Entry(label: "Path".to_string(), value: result.path.display().to_string())
            }
        });
    }

    Ok(())
}

pub fn list(available: bool) -> Result<()> {
    ui::render(element! {
        Header(
            title: "embtool toolchain list".to_string(),
        )
    });

    let installed = toolchain_manager::list()?;

    if installed.is_empty() {
        ui::render(element! {
            StatusLine(
                icon: "·".to_string(),
                message: "No toolchains installed".to_string(),
                variant: StatusVariant::Muted,
            )
        });
    } else {
        for tc in &installed {
            ui::render(element! {
                ToolchainRow(
                    name: tc.name.clone(),
                    gcc_version: tc.gcc_version.clone(),
                    size: format!("{} MB", tc.size_mb),
                    source: String::new(),
                    active: tc.is_active,
                )
            });
        }
    }

    if available {
        println!();
        match config::load().and_then(|c| toolchain_registry::fetch_manifest(&c)) {
            Ok(manifest) => {
                let platform = toolchain_registry::platform_key();

                // Filter: only show toolchains available for current platform
                let available_entries: Vec<_> = manifest.toolchains.iter()
                    .filter(|t| {
                        t.assets.get(platform)
                            .and_then(|a| a.as_ref())
                            .is_some()
                    })
                    .collect();

                if available_entries.is_empty() {
                    ui::render(element! {
                        Section(title: format!("Available for {}", platform)) {
                            StatusLine(
                                icon: "·".to_string(),
                                message: "No toolchains available for this platform".to_string(),
                                variant: StatusVariant::Muted,
                            )
                        }
                    });
                } else {
                    // Check which ones are already installed
                    let installed_names: Vec<String> = toolchain_manager::list()
                        .unwrap_or_default()
                        .iter()
                        .map(|t| t.name.clone())
                        .collect();

                    ui::render(element! {
                        Section(title: format!("Available for {}", platform)) {
                            #(available_entries.iter().map(|entry| {
                                let name = format!("{}-{}", entry.vendor, entry.version);
                                let is_installed = installed_names.contains(&name);
                                let asset = entry.assets.get(platform).unwrap().as_ref().unwrap();
                                let size_mb = asset.size / (1024 * 1024);

                                element! {
                                    View(flex_direction: FlexDirection::Row) {
                                        View(width: 3) {
                                            Text(
                                                content: if is_installed { "✓ ".to_string() } else { "  ".to_string() },
                                                color: Some(Color::Green),
                                            )
                                        }
                                        View(width: 8) {
                                            Text(content: entry.vendor.to_string(), color: Some(Color::Cyan))
                                        }
                                        View(width: 12) {
                                            Text(content: entry.version.to_string(), weight: Weight::Bold)
                                        }
                                        View(width: 14) {
                                            Text(content: format!("gcc {}", entry.gcc), color: Some(Color::DarkGrey))
                                        }
                                        Text(
                                            content: format!("{} MB", size_mb),
                                            color: Some(Color::DarkGrey),
                                        )
                                    }
                                }
                            }))
                        }
                    });
                }
            }
            Err(e) => {
                ui::render(element! {
                    StatusLine(
                        icon: "!".to_string(),
                        message: format!("Could not fetch registry: {}", e),
                        variant: StatusVariant::Warning,
                    )
                });
            }
        }
    }

    Ok(())
}

pub fn use_version(spec: &str) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;
    let tc_path = paths::toolchain_path(&vendor, &version)?;

    if !tc_path.exists() {
        bail!(
            "Toolchain {}-{} is not installed. Run 'embtool toolchain install {}' first.",
            vendor, version, spec
        );
    }

    let mut cfg = config::load()?;
    cfg.toolchain.default = Some(format!("{}-{}", vendor, version));
    config::save(&cfg)?;

    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!("Switched to {} ARM GCC {}", vendor.to_uppercase(), version),
            variant: StatusVariant::Success,
        )
    });

    Ok(())
}

pub fn remove(spec: &str) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;
    let tc_path = paths::toolchain_path(&vendor, &version)?;

    if !tc_path.exists() {
        bail!("Toolchain {}-{} is not installed.", vendor, version);
    }

    let size = dir_size_mb(&tc_path);
    std::fs::remove_dir_all(&tc_path)?;

    let mut cfg = config::load()?;
    let id = format!("{}-{}", vendor, version);
    if cfg.toolchain.default.as_deref() == Some(&id) {
        cfg.toolchain.default = None;
        config::save(&cfg)?;
    }

    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!("Removed {} ARM GCC {} (freed {} MB)", vendor.to_uppercase(), version, size),
            variant: StatusVariant::Success,
        )
    });

    Ok(())
}

fn dir_size_mb(path: &std::path::Path) -> u64 {
    fn walk(path: &std::path::Path) -> u64 {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() { total += walk(&p); }
                else if let Ok(m) = p.metadata() { total += m.len(); }
            }
        }
        total
    }
    walk(path) / (1024 * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec() {
        let (v, ver) = parse_spec("nxp:14.2.1").unwrap();
        assert_eq!(v, "nxp");
        assert_eq!(ver, "14.2.1");
    }

    #[test]
    fn test_parse_spec_invalid() {
        assert!(parse_spec("14.2.1").is_err());
    }
}
