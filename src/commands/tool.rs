use anyhow::Result;
use iocraft::prelude::*;

use crate::core::{clang_provider::ClangTool, tool_manager, tool_provider};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant, ToolchainRow};

pub fn install(tool_name: &str, version: Option<&str>, force: bool) -> Result<()> {
    let tool = ClangTool::from_str(tool_name)?;
    let ver = version.unwrap_or("18");

    ui::render(element! {
        Header(title: format!("embtool tool install {}", tool_name))
    });

    ui::render(element! {
        StatusLine(
            icon: "↓".to_string(),
            message: format!("Installing {} {}...", tool_name, ver),
            variant: StatusVariant::Info,
        )
    });

    let result = tool_manager::install_clang_tool(tool, ver, force)?;

    if result.already_installed {
        ui::render(element! {
            Section(title: tool_name.to_string(), variant: SectionVariant::Success) {
                StatusLine(icon: "✓".to_string(), message: "Already installed".to_string(), variant: StatusVariant::Success)
                Entry(label: "Version".to_string(), value: result.version)
                Entry(label: "Path".to_string(), value: result.path.display().to_string())
            }
        });
    } else {
        ui::render(element! {
            Section(title: tool_name.to_string(), variant: SectionVariant::Success) {
                StatusLine(icon: "✓".to_string(), message: "Installed".to_string(), variant: StatusVariant::Success)
                Entry(label: "Version".to_string(), value: result.version)
                Entry(label: "Size".to_string(), value: format!("{} MB", result.size_mb))
                Entry(label: "Path".to_string(), value: result.path.display().to_string())
            }
        });
    }

    Ok(())
}

pub fn list(tool_name: Option<&str>) -> Result<()> {
    ui::render(element! {
        Header(title: "embtool tool list".to_string())
    });

    let all_tools = tool_manager::list_all_tools()?;

    let filtered: Vec<_> = if let Some(name) = tool_name {
        all_tools.into_iter().filter(|t| t.name == name).collect()
    } else {
        all_tools
    };

    if filtered.is_empty() {
        ui::render(element! {
            StatusLine(
                icon: "·".to_string(),
                message: "No development tools installed".to_string(),
                variant: StatusVariant::Muted,
            )
        });
    } else {
        for tool in &filtered {
            ui::render(element! {
                ToolchainRow(
                    name: format!("{}-{}", tool.name, tool.version),
                    spec: format!("{}:{}", tool.name, tool.version),
                    gcc_version: tool.installed_version.clone(),
                    size: format!("{} MB", tool.size_mb),
                    source: String::new(),
                    active: false,
                )
            });
        }
    }

    Ok(())
}

pub fn list_available(tool_name: &str) -> Result<()> {
    let tool = ClangTool::from_str(tool_name)?;
    let provider = crate::core::clang_provider::ClangProvider::new(tool);
    let versions = crate::core::tool_provider::ToolProvider::available_versions(&provider)?;
    let installed = crate::core::clang_provider::list_installed(tool)?;
    let installed_versions: Vec<String> = installed.iter().map(|(v, _)| v.clone()).collect();

    ui::render(element! {
        Section(title: format!("{} — Available", tool_name)) {
            #(versions.iter().map(|v| {
                let is_installed = installed_versions.contains(v);
                element! {
                    View(flex_direction: FlexDirection::Row) {
                        View(width: 3) {
                            Text(
                                content: if is_installed { "✓ ".to_string() } else { "  ".to_string() },
                                color: Some(Color::Green),
                            )
                        }
                        View(width: 8) {
                            Text(content: v.clone(), weight: Weight::Bold)
                        }
                    }
                }
            }))
        }
    });

    Ok(())
}

pub fn remove(tool_name: &str, version: &str) -> Result<()> {
    let tool = ClangTool::from_str(tool_name)?;
    let (removed_ver, size) = tool_manager::remove_clang_tool(tool, version)?;
    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!("Removed {} {} (freed {} MB)", tool_name, removed_ver, size),
            variant: StatusVariant::Success,
        )
    });
    Ok(())
}
