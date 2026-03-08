use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::core::{config, toolchain_registry};
use crate::utils::{archive, download, paths};

pub struct InstalledToolchain {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub gcc_version: String,
    pub path: PathBuf,
    pub is_active: bool,
    pub size_mb: u64,
}

/// Install a toolchain by vendor and version
pub fn install(vendor: &str, version: &str, force: bool) -> Result<InstalledToolchain> {
    let global_config = config::load()?;
    paths::ensure_dirs()?;

    // 1. Fetch manifest
    let manifest = toolchain_registry::fetch_manifest(&global_config)?;

    // 2. Find toolchain entry
    let entry = toolchain_registry::find_toolchain(&manifest, vendor, version)?;
    let actual_version = entry.version.clone();
    let toolchain_name = format!("{}-{}", entry.vendor, actual_version);

    // 3. Check if already installed
    let tc_path = paths::toolchain_path(&entry.vendor, &actual_version)?;
    let gcc_name = if cfg!(windows) { "arm-none-eabi-gcc.exe" } else { "arm-none-eabi-gcc" };

    if tc_path.join("bin").join(gcc_name).exists() && !force {
        let gcc_ver = get_gcc_version(&tc_path)?;
        return Ok(InstalledToolchain {
            name: toolchain_name,
            vendor: entry.vendor.clone(),
            version: actual_version,
            gcc_version: gcc_ver,
            path: tc_path,
            is_active: false,
            size_mb: 0,
        });
    }

    // 4. Resolve asset for current platform
    let asset = toolchain_registry::resolve_asset(entry)?;

    // 5. Download
    let url = toolchain_registry::download_url(&global_config, asset);
    let cache_dir = paths::cache_dir()?;
    let archive_path = download::download_file(
        &url,
        &cache_dir,
        &asset.file,
        &asset.sha256,
        true,
    )?;

    // 6. Extract
    let tc_dir = paths::toolchains_dir()?;
    archive::extract(&archive_path, &tc_dir, &toolchain_name)?;

    // 7. Verify
    let gcc_ver = get_gcc_version(&tc_path)
        .context("Toolchain installed but gcc verification failed")?;

    // 8. Set as default if first install
    let mut cfg = config::load()?;
    if cfg.toolchain.default.is_none() {
        cfg.toolchain.default = Some(toolchain_name.clone());
        config::save(&cfg)?;
    }

    let size_mb = dir_size_mb(&tc_path);

    Ok(InstalledToolchain {
        name: toolchain_name.clone(),
        vendor: entry.vendor.clone(),
        version: actual_version,
        gcc_version: gcc_ver,
        path: tc_path,
        is_active: cfg.toolchain.default.as_deref() == Some(&toolchain_name.clone()),
        size_mb,
    })
}

/// List all installed toolchains
pub fn list() -> Result<Vec<InstalledToolchain>> {
    let tc_dir = paths::toolchains_dir()?;
    let cfg = config::load()?;

    if !tc_dir.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&tc_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        let gcc_name = if cfg!(windows) { "arm-none-eabi-gcc.exe" } else { "arm-none-eabi-gcc" };
        if !path.join("bin").join(gcc_name).exists() {
            continue;
        }

        let gcc_version = get_gcc_version(&path).unwrap_or_else(|_| "?".to_string());
        let (vendor, version) = parse_toolchain_name(&name);
        let is_active = cfg.toolchain.default.as_deref() == Some(&name);
        let size_mb = dir_size_mb(&path);

        result.push(InstalledToolchain {
            name: name.clone(),
            vendor,
            version,
            gcc_version,
            path,
            is_active,
            size_mb,
        });
    }

    Ok(result)
}

/// Get gcc version string from a toolchain path
fn get_gcc_version(tc_path: &std::path::Path) -> Result<String> {
    let gcc_name = if cfg!(windows) { "arm-none-eabi-gcc.exe" } else { "arm-none-eabi-gcc" };
    let gcc = tc_path.join("bin").join(gcc_name);

    let output = std::process::Command::new(&gcc)
        .arg("--version")
        .output()
        .with_context(|| format!("Failed to run {}", gcc.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse: "arm-none-eabi-gcc (Arm GNU Toolchain 14.2.Rel1 ...) 14.2.1 20241119"
    // Look for version pattern: digits.digits.digits after closing paren
    stdout
        .lines()
        .next()
        .and_then(|line| {
            // Find text after last ')' if present
            let search = if let Some(pos) = line.rfind(')') {
                &line[pos + 1..]
            } else {
                line
            };
            // Find first token matching X.Y.Z pattern
            search.split_whitespace()
                .find(|w| {
                    let parts: Vec<&str> = w.split('.').collect();
                    parts.len() >= 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
                })
        })
        .map(|v| v.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not parse gcc version"))
}

fn parse_toolchain_name(name: &str) -> (String, String) {
    if let Some((vendor, version)) = name.split_once('-') {
        (vendor.to_string(), version.to_string())
    } else {
        (name.to_string(), String::new())
    }
}

fn dir_size_mb(path: &std::path::Path) -> u64 {
    dir_size(path) / (1024 * 1024)
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}
