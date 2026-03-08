use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::core::cmake_provider::{self, CmakeProvider};
use crate::core::clang_provider::{self, ClangProvider, ClangTool};
use crate::core::tool_provider::{self, ArchiveType, InstallResult, ToolProvider};
use crate::utils::{download, paths};

/// Install CMake
pub fn install_cmake(version_spec: &str, force: bool) -> Result<InstallResult> {
    let version = CmakeProvider::resolve_version(version_spec)?;
    let provider = CmakeProvider;

    let install_dir = provider.install_dir(&version)?;

    // Check if already installed
    if !force {
        if let Ok(ver) = provider.verify_install(&install_dir) {
            return Ok(InstallResult {
                name: "cmake".to_string(),
                version: ver,
                path: install_dir,
                size_mb: 0,
                already_installed: true,
            });
        }
    }

    // Download
    let url = provider.download_url(&version)?;
    let filename = provider.archive_filename(&version)?;
    let cache_dir = paths::cache_dir()?;

    // No SHA256 for CMake GitHub releases easily accessible,
    // so we download via HTTPS and trust the source
    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to download CMake from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed: HTTP {} from {}", response.status(), url);
    }

    let total_size = response.content_length().unwrap_or(0);
    let archive_path = cache_dir.join(&filename);
    std::fs::create_dir_all(&cache_dir)?;

    // Download with progress
    {
        use std::io::Write;
        let pb = if total_size > 0 && !crate::core::config::is_ci() {
            let pb = indicatif::ProgressBar::new(total_size);
            pb.set_style(
                indicatif::ProgressStyle::default_bar()
                    .template("   {bar:40.cyan/dim} {percent}% ({bytes}/{total_bytes})")
                    .unwrap()
                    .progress_chars("━╸─"),
            );
            Some(pb)
        } else {
            if total_size > 0 {
                eprintln!("   Downloading {} ({:.1} MB)...", filename, total_size as f64 / 1_048_576.0);
            }
            None
        };

        let mut file = std::fs::File::create(&archive_path)?;
        let mut downloaded = 0u64;
        let mut reader = response;

        loop {
            let mut buf = [0u8; 8192];
            let n = std::io::Read::read(&mut reader, &mut buf)?;
            if n == 0 { break; }
            file.write_all(&buf[..n])?;
            downloaded += n as u64;
            if let Some(ref pb) = pb { pb.set_position(downloaded); }
        }
        if let Some(pb) = pb { pb.finish_and_clear(); }
    }

    // Extract
    match provider.archive_type() {
        ArchiveType::TarGz => cmake_provider::extract_tar_gz(&archive_path, &install_dir)?,
        ArchiveType::Zip => cmake_provider::extract_zip(&archive_path, &install_dir)?,
        _ => unreachable!(),
    }

    // Verify
    let installed_version = provider.verify_install(&install_dir)
        .context("CMake installed but verification failed")?;

    let size_mb = tool_provider::dir_size_mb(&install_dir);

    Ok(InstallResult {
        name: "cmake".to_string(),
        version: installed_version,
        path: install_dir,
        size_mb,
        already_installed: false,
    })
}

/// Install a clang tool (clang-format or clang-tidy)
pub fn install_clang_tool(tool: ClangTool, version: &str, force: bool) -> Result<InstallResult> {
    let provider = ClangProvider::new(tool);
    let install_dir = provider.install_dir(version)?;
    let bin_name = if cfg!(windows) {
        format!("{}.exe", tool.name())
    } else {
        tool.name().to_string()
    };

    // Check if already installed
    if !force {
        if let Ok(ver) = provider.verify_install(&install_dir) {
            return Ok(InstallResult {
                name: tool.name().to_string(),
                version: ver,
                path: install_dir,
                size_mb: 0,
                already_installed: true,
            });
        }
    }

    // Download
    let url = provider.download_url(version)?;
    clang_provider::install_binary(&url, &install_dir, &bin_name)?;

    // Verify
    let installed_version = provider.verify_install(&install_dir)
        .context(format!("{} installed but verification failed", tool.name()))?;

    let size_mb = tool_provider::file_size_mb(&install_dir.join(&bin_name));

    Ok(InstallResult {
        name: tool.name().to_string(),
        version: installed_version,
        path: install_dir,
        size_mb,
        already_installed: false,
    })
}

/// Installed tool info
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub installed_version: String,
    pub path: PathBuf,
    pub size_mb: u64,
}

/// List all installed development tools
pub fn list_all_tools() -> Result<Vec<ToolInfo>> {
    let mut tools = Vec::new();

    // CMake
    for (version, path) in cmake_provider::list_installed()? {
        let provider = CmakeProvider;
        let installed_ver = provider.verify_install(&path).unwrap_or_else(|_| "?".to_string());
        let size_mb = tool_provider::dir_size_mb(&path);
        tools.push(ToolInfo {
            name: "cmake".to_string(),
            version: version.clone(),
            installed_version: installed_ver,
            path,
            size_mb,
        });
    }

    // clang-format
    for (version, path) in clang_provider::list_installed(ClangTool::Format)? {
        let provider = ClangProvider::new(ClangTool::Format);
        let installed_ver = provider.verify_install(&path).unwrap_or_else(|_| "?".to_string());
        let size_mb = tool_provider::file_size_mb(
            &path.join(if cfg!(windows) { "clang-format.exe" } else { "clang-format" })
        );
        tools.push(ToolInfo {
            name: "clang-format".to_string(),
            version,
            installed_version: installed_ver,
            path,
            size_mb,
        });
    }

    // clang-tidy
    for (version, path) in clang_provider::list_installed(ClangTool::Tidy)? {
        let provider = ClangProvider::new(ClangTool::Tidy);
        let installed_ver = provider.verify_install(&path).unwrap_or_else(|_| "?".to_string());
        let size_mb = tool_provider::file_size_mb(
            &path.join(if cfg!(windows) { "clang-tidy.exe" } else { "clang-tidy" })
        );
        tools.push(ToolInfo {
            name: "clang-tidy".to_string(),
            version,
            installed_version: installed_ver,
            path,
            size_mb,
        });
    }

    Ok(tools)
}

/// Remove a CMake version
pub fn remove_cmake(version: &str) -> Result<(String, u64)> {
    let version = CmakeProvider::resolve_version(version)?;
    let provider = CmakeProvider;
    let dir = provider.install_dir(&version)?;
    if !dir.exists() {
        anyhow::bail!("CMake {} is not installed.", version);
    }
    let size = tool_provider::dir_size_mb(&dir);
    std::fs::remove_dir_all(&dir)?;
    Ok((version, size))
}

/// Remove a clang tool version
pub fn remove_clang_tool(tool: ClangTool, version: &str) -> Result<(String, u64)> {
    let provider = ClangProvider::new(tool);
    let dir = provider.install_dir(version)?;
    if !dir.exists() {
        anyhow::bail!("{} version {} is not installed.", tool.name(), version);
    }
    let bin_name = if cfg!(windows) { format!("{}.exe", tool.name()) } else { tool.name().to_string() };
    let size = tool_provider::file_size_mb(&dir.join(&bin_name));
    std::fs::remove_dir_all(&dir)?;
    Ok((version.to_string(), size))
}
