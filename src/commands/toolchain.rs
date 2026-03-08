use anyhow::{bail, Result};
use iocraft::prelude::*;

use crate::ui::{self, Header, Section, SectionVariant, StatusLine, StatusVariant, ToolchainRow};
use crate::utils::paths;

/// Parse "vendor:version" spec, e.g. "nxp:14.2" or "nxp:14.2.1"
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

pub fn install(spec: &str, _force: bool) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;

    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(
                title: "embtool toolchain install".to_string(),
            )
            Section(title: format!("{} ARM GCC {}", vendor.to_uppercase(), version)) {
                StatusLine(
                    icon: "→".to_string(),
                    message: "Not yet implemented".to_string(),
                    variant: StatusVariant::Muted,
                )
            }
        }
    });

    Ok(())
}

pub fn list(available: bool) -> Result<()> {
    let tc_dir = paths::toolchains_dir()?;

    ui::render(element! {
        Header(
            title: "embtool toolchain list".to_string(),
        )
    });

    if !tc_dir.exists() {
        ui::render(element! {
            StatusLine(
                icon: "·".to_string(),
                message: "No toolchains installed".to_string(),
                variant: StatusVariant::Muted,
            )
        });
        return Ok(());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&tc_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut found = false;

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();

        let gcc = entry.path().join("bin").join("arm-none-eabi-gcc");
        let gcc_exe = entry.path().join("bin").join("arm-none-eabi-gcc.exe");
        if gcc.exists() || gcc_exe.exists() {
            found = true;

            // Get gcc version
            let gcc_path = if gcc.exists() { &gcc } else { &gcc_exe };
            let gcc_ver = std::process::Command::new(gcc_path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| {
                    s.lines()
                        .next()
                        .and_then(|l| l.split_whitespace().last())
                        .map(|v| v.to_string())
                })
                .unwrap_or_else(|| "?".to_string());

            // Get size
            let size = fs_size_mb(&entry.path());

            ui::render(element! {
                ToolchainRow(
                    name: name_str,
                    gcc_version: gcc_ver,
                    size: format!("{} MB", size),
                    source: String::new(),
                    active: false,
                )
            });
        }
    }

    if !found {
        ui::render(element! {
            StatusLine(
                icon: "·".to_string(),
                message: "No toolchains installed".to_string(),
                variant: StatusVariant::Muted,
            )
        });
    }

    if available {
        println!();
        ui::render(element! {
            Section(title: "Available (remote)".to_string()) {
                StatusLine(
                    icon: "→".to_string(),
                    message: "Not yet implemented".to_string(),
                    variant: StatusVariant::Muted,
                )
            }
        });
    }

    Ok(())
}

pub fn use_version(spec: &str) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;
    let tc_path = paths::toolchain_path(&vendor, &version)?;

    if !tc_path.exists() {
        bail!(
            "Toolchain {}-{} is not installed. Run 'embtool toolchain install {}' first.",
            vendor,
            version,
            spec
        );
    }

    let mut config = crate::core::config::load()?;
    config.toolchain.default = Some(format!("{}-{}", vendor, version));
    crate::core::config::save(&config)?;

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

    let size = fs_size_mb(&tc_path);
    std::fs::remove_dir_all(&tc_path)?;

    let mut config = crate::core::config::load()?;
    let id = format!("{}-{}", vendor, version);
    if config.toolchain.default.as_deref() == Some(&id) {
        config.toolchain.default = None;
        crate::core::config::save(&config)?;
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

fn fs_size_mb(path: &std::path::Path) -> u64 {
    walkdir_size(path) / (1024 * 1024)
}

fn walkdir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += walkdir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec() {
        let (v, ver) = parse_spec("nxp:14.2.1").unwrap();
        assert_eq!(v, "nxp");
        assert_eq!(ver, "14.2.1");

        let (v, ver) = parse_spec("stm:13.3").unwrap();
        assert_eq!(v, "stm");
        assert_eq!(ver, "13.3");
    }

    #[test]
    fn test_parse_spec_invalid() {
        assert!(parse_spec("14.2.1").is_err());
        assert!(parse_spec("").is_err());
    }
}
