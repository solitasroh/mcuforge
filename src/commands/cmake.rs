use anyhow::Result;
use iocraft::prelude::*;

use crate::core::{cmake_provider::CmakeProvider, tool_manager, tool_provider::{self, ToolProvider}};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant, ToolchainRow};

pub fn install(version: Option<&str>, force: bool) -> Result<()> {
    let spec = version.unwrap_or(crate::core::cmake_provider::DEFAULT_CMAKE_VERSION);

    ui::render(element! {
        Header(title: "embtool cmake install".to_string())
    });

    let resolved = CmakeProvider::resolve_version(spec)?;
    ui::render(element! {
        StatusLine(
            icon: "↓".to_string(),
            message: format!("Installing CMake {}...", resolved),
            variant: StatusVariant::Info,
        )
    });

    let result = tool_manager::install_cmake(spec, force)?;

    if result.already_installed {
        ui::render(element! {
            Section(title: "cmake".to_string(), variant: SectionVariant::Success) {
                StatusLine(icon: "✓".to_string(), message: "Already installed".to_string(), variant: StatusVariant::Success)
                Entry(label: "Version".to_string(), value: result.version)
                Entry(label: "Path".to_string(), value: result.path.display().to_string())
            }
        });
    } else {
        ui::render(element! {
            Section(title: "cmake".to_string(), variant: SectionVariant::Success) {
                StatusLine(icon: "✓".to_string(), message: "Installed".to_string(), variant: StatusVariant::Success)
                Entry(label: "Version".to_string(), value: result.version)
                Entry(label: "Size".to_string(), value: format!("{} MB", result.size_mb))
                Entry(label: "Path".to_string(), value: result.path.display().to_string())
            }
        });
    }

    Ok(())
}

pub fn list(available: bool) -> Result<()> {
    ui::render(element! {
        Header(title: "embtool cmake list".to_string())
    });

    let installed = crate::core::cmake_provider::list_installed()?;

    if installed.is_empty() {
        ui::render(element! {
            StatusLine(icon: "·".to_string(), message: "No CMake versions installed".to_string(), variant: StatusVariant::Muted)
        });
    } else {
        for (version, path) in &installed {
            let provider = CmakeProvider;
            let ver = crate::core::tool_provider::ToolProvider::verify_install(&provider, path)
                .unwrap_or_else(|_| "?".to_string());
            let size = tool_provider::dir_size_mb(path);
            ui::render(element! {
                ToolchainRow(
                    name: format!("cmake-{}", version),
                    spec: format!("cmake:{}", &version.split('.').take(2).collect::<Vec<_>>().join(".")),
                    gcc_version: ver,
                    size: format!("{} MB", size),
                    source: String::new(),
                    active: false,
                )
            });
        }
    }

    if available {
        println!();
        let all_versions = CmakeProvider.available_versions()?;
        // Show top 20 grouped by minor
        let mut shown = Vec::new();
        let mut last_minor = String::new();
        for v in &all_versions {
            let parts: Vec<&str> = v.as_str().split('.').collect();
            let minor = format!("{}.{}", parts[0], parts[1]);
            if minor != last_minor {
                let is_installed = installed.iter().any(|(iv, _)| iv == v);
                shown.push((v.clone(), is_installed));
                last_minor = minor;
            }
            if shown.len() >= 15 { break; }
        }

        ui::render(element! {
            Section(title: "Available (latest per minor)".to_string()) {
                #(shown.iter().map(|(v, installed): &(String, bool)| {
                    element! {
                        View(flex_direction: FlexDirection::Row) {
                            View(width: 3) {
                                Text(
                                    content: if *installed { "✓ ".to_string() } else { "  ".to_string() },
                                    color: Some(Color::Green),
                                )
                            }
                            View(width: 12) {
                                Text(content: v.clone(), weight: Weight::Bold)
                            }
                        }
                    }
                }))
            }
        });
    }

    Ok(())
}

pub fn remove(version: &str) -> Result<()> {
    let (removed_ver, size) = tool_manager::remove_cmake(version)?;
    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!("Removed CMake {} (freed {} MB)", removed_ver, size),
            variant: StatusVariant::Success,
        )
    });
    Ok(())
}
