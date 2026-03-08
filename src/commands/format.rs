use std::process::Command;

use anyhow::{bail, Result};
use iocraft::prelude::*;

use crate::core::project;
use crate::ui::{self, Header, StatusLine, StatusVariant};
use crate::utils::paths;

pub fn run(check: bool) -> Result<()> {
    let project_path = project::find_project()?;
    let project_dir = project_path.parent().unwrap();
    let proj = project::load(&project_path)?;

    // Find clang-format binary
    let clang_format = find_clang_format(&proj)?;

    // Find source files
    let sources = find_sources(project_dir)?;

    if sources.is_empty() {
        ui::render(element! {
            StatusLine(
                icon: "·".to_string(),
                message: "No source files found".to_string(),
                variant: StatusVariant::Muted,
            )
        });
        return Ok(());
    }

    let mode = if check { "Checking" } else { "Formatting" };
    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(title: format!("embtool format{}", if check { " --check" } else { "" }))
            StatusLine(
                icon: "⚙".to_string(),
                message: format!("{} {} files...", mode, sources.len()),
                variant: StatusVariant::Info,
            )
        }
    });

    let mut cmd = Command::new(&clang_format);

    if check {
        cmd.arg("--dry-run").arg("--Werror");
    } else {
        cmd.arg("-i"); // in-place
    }

    // Use .clang-format style if exists, else LLVM
    if project_dir.join(".clang-format").exists() {
        cmd.arg("--style=file");
    }

    for src in &sources {
        cmd.arg(src);
    }

    cmd.current_dir(project_dir);
    let output = cmd.output()?;

    if output.status.success() {
        let msg = if check {
            format!("All {} files formatted correctly ✓", sources.len())
        } else {
            format!("Formatted {} files ✓", sources.len())
        };
        println!();
        ui::render(element! {
            StatusLine(icon: "✓".to_string(), message: msg, variant: StatusVariant::Success)
        });
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if check {
            println!();
            ui::render(element! {
                StatusLine(
                    icon: "✗".to_string(),
                    message: "Formatting issues found:".to_string(),
                    variant: StatusVariant::Error,
                )
            });
            eprintln!("{}", stderr);
            bail!("Format check failed");
        } else {
            // clang-format -i rarely fails
            eprintln!("{}", stderr);
            bail!("clang-format failed");
        }
    }
}

fn find_clang_format(proj: &project::ProjectConfig) -> Result<std::path::PathBuf> {
    // Check project-configured version
    if let Some(ref cf) = proj.tools.clang_format {
        let embtool_home = paths::embtool_home()?;
        let tool_dir = embtool_home.join("tools").join("clang-format").join(&cf.version);
        let bin_name = if cfg!(windows) { "clang-format.exe" } else { "clang-format" };
        let bin = tool_dir.join(bin_name);
        if bin.exists() {
            return Ok(bin);
        }
    }

    // Fallback to system
    if let Ok(output) = Command::new("clang-format").arg("--version").output() {
        if output.status.success() {
            return Ok(std::path::PathBuf::from("clang-format"));
        }
    }

    bail!("clang-format not found. Run 'embtool tool install clang-format <version>' or 'embtool setup'.")
}

fn find_sources(project_dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut sources = Vec::new();
    let extensions = ["c", "h", "cpp", "hpp", "cc", "hh"];

    walk_dir(project_dir, &extensions, &mut sources)?;
    sources.sort();
    Ok(sources)
}

fn walk_dir(
    dir: &std::path::Path,
    extensions: &[&str],
    results: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    let skip_dirs = ["build", ".git", "cmake-build-debug", "cmake-build-release"];

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if !skip_dirs.contains(&name.as_str()) {
                walk_dir(&path, extensions, results)?;
            }
        } else if let Some(ext) = path.extension() {
            if extensions.iter().any(|e| *e == ext.to_string_lossy().as_ref()) {
                results.push(path);
            }
        }
    }

    Ok(())
}
