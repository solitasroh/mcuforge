use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};
use iocraft::prelude::*;

use crate::core::project;
use crate::ui::{self, Header, StatusLine, StatusVariant};
use crate::utils::paths;

pub fn run(fix: bool) -> Result<()> {
    let project_path = project::find_project()?;
    let project_dir = project_path.parent().unwrap();
    let proj = project::load(&project_path)?;

    // Find clang-tidy binary
    let clang_tidy = find_clang_tidy(&proj)?;

    // Check compile_commands.json
    let cc_path = project_dir.join("build").join("compile_commands.json");
    let cc_root = project_dir.join("compile_commands.json");

    if !cc_path.exists() && !cc_root.exists() {
        bail!("compile_commands.json not found. Run 'mcuforge build' first.");
    }

    let compile_db = if cc_root.exists() {
        project_dir.to_path_buf()
    } else {
        project_dir.join("build")
    };

    // Find source files
    let sources = find_c_sources(project_dir)?;

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

    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(title: format!("embtool lint{}", if fix { " --fix" } else { "" }))
            StatusLine(
                icon: "⚙".to_string(),
                message: format!("Analyzing {} files...", sources.len()),
                variant: StatusVariant::Info,
            )
        }
    });

    let mut cmd = Command::new(&clang_tidy);
    cmd.arg(format!("-p={}", compile_db.display()));

    if fix {
        cmd.arg("--fix");
    }

    // Add ARM toolchain sysroot for cross-compilation header resolution
    if let Ok(proj) = project::load(&project_path) {
        let tc_path = crate::utils::paths::toolchain_path(
            &proj.toolchain.vendor,
            &proj.toolchain.version,
        );
        if let Ok(tc) = tc_path {
            let sysroot = tc.join("arm-none-eabi").join("include");
            if sysroot.exists() {
                cmd.arg(format!("--extra-arg=-isystem{}", sysroot.display()));
            }
        }
    }

    // Use .clang-tidy if exists
    if project_dir.join(".clang-tidy").exists() {
        cmd.arg("--config-file").arg(project_dir.join(".clang-tidy"));
    }

    for src in &sources {
        cmd.arg(src);
    }

    cmd.current_dir(project_dir);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // clang-tidy outputs diagnostics to stdout
    if !stdout.is_empty() {
        println!();
        print!("{}", stdout);
    }
    if !stderr.is_empty() && !stderr.contains("warnings generated") {
        eprint!("{}", stderr);
    }

    println!();
    if output.status.success() {
        ui::render(element! {
            StatusLine(
                icon: "✓".to_string(),
                message: format!("Lint passed ({} files)", sources.len()),
                variant: StatusVariant::Success,
            )
        });
        Ok(())
    } else {
        ui::render(element! {
            StatusLine(
                icon: "!".to_string(),
                message: "Lint issues found".to_string(),
                variant: StatusVariant::Warning,
            )
        });
        // Don't bail — lint warnings are informational
        Ok(())
    }
}

fn find_clang_tidy(proj: &project::ProjectConfig) -> Result<std::path::PathBuf> {
    if let Some(ref ct) = proj.tools.clang_tidy {
        let embtool_home = paths::embtool_home()?;
        let tool_dir = embtool_home.join("tools").join("clang-tidy").join(&ct.version);
        let bin_name = if cfg!(windows) { "clang-tidy.exe" } else { "clang-tidy" };
        let bin = tool_dir.join(bin_name);
        if bin.exists() {
            return Ok(bin);
        }
    }

    if let Ok(output) = Command::new("clang-tidy").arg("--version").output() {
        if output.status.success() {
            return Ok(std::path::PathBuf::from("clang-tidy"));
        }
    }

    bail!("clang-tidy not found. Run 'mcuforge tool install clang-tidy <version>' or 'mcuforge setup'.")
}

fn find_c_sources(project_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut sources = Vec::new();
    // Only lint .c and .cpp (not headers — clang-tidy analyzes them via includes)
    let extensions = ["c", "cpp", "cc"];
    walk_dir(project_dir, &extensions, &mut sources)?;
    sources.sort();
    Ok(sources)
}

fn walk_dir(
    dir: &Path,
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
