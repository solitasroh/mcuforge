use anyhow::{Context, Result};
use iocraft::prelude::*;

use crate::core::{config, project, tool_manager, cmake_provider, clang_provider};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant};
use crate::utils::paths;

pub fn run(ci: bool, force: bool) -> Result<()> {
    // 1. Ensure ~/.embtool directories exist
    paths::ensure_dirs().context("Failed to create embtool directories")?;

    // 2. Find and load embtool.toml
    let project_path = project::find_project()?;
    let project_dir = project_path.parent().unwrap();
    let proj = project::load(&project_path)?;

    // 3. Load global config
    let _global_config = config::load()?;
    let _is_ci = ci || config::is_ci();

    // 4. Render project info
    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(
                title: "embtool setup".to_string(),
                subtitle: Some(project_dir.display().to_string()),
            )
            Section(title: "Project".to_string()) {
                Entry(label: "Name".to_string(), value: proj.project.name.clone())
                Entry(label: "MCU".to_string(), value: format!("{} ({})", proj.target.mcu, proj.target.core))
                Entry(label: "Toolchain".to_string(), value: proj.toolchain.id())
                Entry(label: "CMake".to_string(), value: proj.cmake.version.clone())
            }
        }
    });

    println!();

    // 5. Check / install toolchain
    let tc_ok = check_toolchain(&proj, force)?;

    // 6. Check / install CMake
    let cmake_ok = check_cmake(&proj, force)?;

    // 7. Check / install clang-format (if configured)
    let format_ok = if let Some(ref cf) = proj.tools.clang_format {
        check_clang_tool("clang-format", &cf.version, force)?
    } else {
        true
    };

    // 8. Check / install clang-tidy (if configured)
    let tidy_ok = if let Some(ref ct) = proj.tools.clang_tidy {
        check_clang_tool("clang-tidy", &ct.version, force)?
    } else {
        true
    };

    // 9. Generate arm-toolchain.cmake
    let cmake_path = project_dir.join("arm-toolchain.cmake");
    if cmake_path.exists() && !force {
        ui::render(element! {
            StatusLine(
                icon: "·".to_string(),
                message: "arm-toolchain.cmake exists (use --force to regenerate)".to_string(),
                variant: StatusVariant::Muted,
            )
        });
    } else {
        generate_toolchain_cmake(&cmake_path, &proj)?;
        ui::render(element! {
            StatusLine(
                icon: "✓".to_string(),
                message: "Generated arm-toolchain.cmake".to_string(),
                variant: StatusVariant::Success,
            )
        });
    }

    // 10. Summary
    println!();
    let all_ok = tc_ok && cmake_ok && format_ok && tidy_ok;
    if all_ok {
        ui::render(element! {
            Section(title: "Ready".to_string(), variant: SectionVariant::Success) {
                StatusLine(
                    icon: "✓".to_string(),
                    message: "All tools installed. Run 'embtool build' to build.".to_string(),
                    variant: StatusVariant::Success,
                )
            }
        });
    } else {
        ui::render(element! {
            Section(title: "Incomplete".to_string()) {
                StatusLine(
                    icon: "!".to_string(),
                    message: "Some tools could not be installed. Check errors above.".to_string(),
                    variant: StatusVariant::Warning,
                )
            }
        });
    }

    Ok(())
}

fn check_toolchain(proj: &project::ProjectConfig, force: bool) -> Result<bool> {
    let tc_path = paths::toolchain_path(&proj.toolchain.vendor, &proj.toolchain.version)?;
    let gcc_name = if cfg!(windows) { "arm-none-eabi-gcc.exe" } else { "arm-none-eabi-gcc" };

    if tc_path.join("bin").join(gcc_name).exists() && !force {
        ui::render(element! {
            StatusLine(
                icon: "✓".to_string(),
                message: format!("Toolchain {} ready", proj.toolchain.id()),
                variant: StatusVariant::Success,
            )
        });
        Ok(true)
    } else {
        ui::render(element! {
            StatusLine(
                icon: "↓".to_string(),
                message: format!("Installing toolchain {}...", proj.toolchain.id()),
                variant: StatusVariant::Info,
            )
        });
        // Auto-install
        let spec = format!("{}:{}", proj.toolchain.vendor, proj.toolchain.version);
        match crate::commands::toolchain::install(&spec, false) {
            Ok(()) => Ok(true),
            Err(e) => {
                ui::render(element! {
                    StatusLine(
                        icon: "✗".to_string(),
                        message: format!("Toolchain install failed: {}", e),
                        variant: StatusVariant::Error,
                    )
                });
                Ok(false)
            }
        }
    }
}

fn check_cmake(proj: &project::ProjectConfig, force: bool) -> Result<bool> {
    let resolved = cmake_provider::CmakeProvider::resolve_version(&proj.cmake.version)?;

    let installed = cmake_provider::list_installed()?;
    let already = installed.iter().any(|(v, _)| v == &resolved);

    if already && !force {
        ui::render(element! {
            StatusLine(
                icon: "✓".to_string(),
                message: format!("CMake {} ready", resolved),
                variant: StatusVariant::Success,
            )
        });
        Ok(true)
    } else {
        ui::render(element! {
            StatusLine(
                icon: "↓".to_string(),
                message: format!("Installing CMake {}...", resolved),
                variant: StatusVariant::Info,
            )
        });
        match tool_manager::install_cmake(&proj.cmake.version, false) {
            Ok(_) => Ok(true),
            Err(e) => {
                ui::render(element! {
                    StatusLine(
                        icon: "✗".to_string(),
                        message: format!("CMake install failed: {}", e),
                        variant: StatusVariant::Error,
                    )
                });
                Ok(false)
            }
        }
    }
}

fn check_clang_tool(tool_name: &str, version: &str, force: bool) -> Result<bool> {
    let tool = if tool_name == "clang-format" {
        clang_provider::ClangTool::Format
    } else {
        clang_provider::ClangTool::Tidy
    };

    let installed = clang_provider::list_installed(tool)?;
    let already = installed.iter().any(|(v, _)| v.starts_with(version));

    if already && !force {
        ui::render(element! {
            StatusLine(
                icon: "✓".to_string(),
                message: format!("{} {} ready", tool_name, version),
                variant: StatusVariant::Success,
            )
        });
        Ok(true)
    } else {
        ui::render(element! {
            StatusLine(
                icon: "↓".to_string(),
                message: format!("Installing {} {}...", tool_name, version),
                variant: StatusVariant::Info,
            )
        });
        match tool_manager::install_clang_tool(tool, version, false) {
            Ok(_) => Ok(true),
            Err(e) => {
                ui::render(element! {
                    StatusLine(
                        icon: "✗".to_string(),
                        message: format!("{} install failed: {}", tool_name, e),
                        variant: StatusVariant::Error,
                    )
                });
                Ok(false)
            }
        }
    }
}

fn generate_toolchain_cmake(
    path: &std::path::Path,
    proj: &project::ProjectConfig,
) -> Result<()> {
    let content = crate::core::template::generate_toolchain_cmake(
        &proj.toolchain.vendor,
        &proj.toolchain.version,
    );
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
