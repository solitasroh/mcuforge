use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::config::GlobalConfig;
use crate::utils::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionsManifest {
    pub schema_version: u32,
    pub latest: HashMap<String, String>,
    pub toolchains: Vec<ToolchainEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainEntry {
    pub version: String,
    pub vendor: String,
    pub gcc: String,
    pub source: String,
    pub date: String,
    #[serde(default)]
    pub includes: Vec<String>,
    pub assets: HashMap<String, Option<AssetInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub file: String,
    pub size: u64,
    pub sha256: String,
}

/// Current platform key for asset lookup
pub fn platform_key() -> &'static str {
    if cfg!(target_os = "windows") {
        "win-x64"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "linux-x64"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "linux-aarch64"
    } else {
        "unknown"
    }
}

/// Fetch versions.json from registry (R2) or cache
pub fn fetch_manifest(config: &GlobalConfig) -> Result<VersionsManifest> {
    let cache_path = paths::cached_versions_path()?;

    // Check cache freshness
    if let Ok(meta) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = meta.modified() {
            let age = SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default();
            let ttl = std::time::Duration::from_secs(config.registry.cache_ttl_hours as u64 * 3600);
            if age < ttl {
                if let Ok(manifest) = load_cached_manifest(&cache_path) {
                    return Ok(manifest);
                }
            }
        }
    }

    // Download from registry
    let url = format!("{}/versions.json", config.registry.url.trim_end_matches('/'));
    let response = reqwest::blocking::get(&url)
        .with_context(|| format!("Failed to fetch {}", url))?;

    if !response.status().is_success() {
        // Fall back to cache if available
        if cache_path.exists() {
            eprintln!("Warning: Could not fetch remote registry, using cached version");
            return load_cached_manifest(&cache_path);
        }
        bail!("Failed to fetch versions.json: HTTP {}", response.status());
    }

    let body = response.text()?;
    let manifest: VersionsManifest = serde_json::from_str(&body)
        .context("Failed to parse versions.json")?;

    // Cache it
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cache_path, &body)?;

    Ok(manifest)
}

fn load_cached_manifest(path: &Path) -> Result<VersionsManifest> {
    let content = std::fs::read_to_string(path)?;
    let manifest: VersionsManifest = serde_json::from_str(&content)?;
    Ok(manifest)
}

/// Find a toolchain entry by vendor and version
pub fn find_toolchain<'a>(
    manifest: &'a VersionsManifest,
    vendor: &str,
    version: &str,
) -> Result<&'a ToolchainEntry> {
    // Try exact match first
    if let Some(entry) = manifest.toolchains.iter().find(|t| {
        t.vendor == vendor && t.version == version
    }) {
        return Ok(entry);
    }

    // Try prefix match (e.g., "14.2" → "14.2.1")
    if let Some(entry) = manifest.toolchains.iter().find(|t| {
        t.vendor == vendor && t.version.starts_with(version)
    }) {
        return Ok(entry);
    }

    // Handle "latest"
    if version == "latest" {
        if let Some(latest_ver) = manifest.latest.get(vendor) {
            return find_toolchain(manifest, vendor, latest_ver);
        }
    }

    bail!(
        "Toolchain {}:{} not found. Run 'embtool toolchain list --available' to see options.",
        vendor, version
    );
}

/// Resolve the asset for current platform
pub fn resolve_asset(entry: &ToolchainEntry) -> Result<&AssetInfo> {
    let key = platform_key();
    match entry.assets.get(key) {
        Some(Some(asset)) => Ok(asset),
        _ => bail!(
            "No {} binary available for {}-{}",
            key, entry.vendor, entry.version
        ),
    }
}

/// Build download URL for an asset
pub fn download_url(config: &GlobalConfig, asset: &AssetInfo) -> String {
    let base = if config.mirror.enabled && !config.mirror.url.is_empty() {
        &config.mirror.url
    } else {
        &config.registry.url
    };
    format!("{}/{}", base.trim_end_matches('/'), asset.file)
}

/// List all available versions
pub fn available_versions(manifest: &VersionsManifest) -> Vec<(&str, &str, &str)> {
    manifest.toolchains.iter().map(|t| {
        (t.vendor.as_str(), t.version.as_str(), t.gcc.as_str())
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> VersionsManifest {
        let json = r#"{
            "schema_version": 1,
            "latest": {"nxp": "14.2.1", "stm": "13.3.1"},
            "toolchains": [
                {
                    "version": "14.2.1", "vendor": "nxp", "gcc": "14.2.1",
                    "source": "MCUXpresso", "date": "2025-06",
                    "includes": ["redlib"],
                    "assets": {
                        "linux-x64": {"file": "nxp-14.2.1-linux-x64.7z", "size": 143668877, "sha256": "abc123"},
                        "win-x64": {"file": "nxp-14.2.1-win-x64.7z", "size": 122777837, "sha256": "def456"}
                    }
                },
                {
                    "version": "13.3.1", "vendor": "stm", "gcc": "13.3.1",
                    "source": "STM32CubeIDE", "date": "2025-09",
                    "assets": {
                        "win-x64": {"file": "stm-13.3.1-win-x64.7z", "size": 152698895, "sha256": "ghi789"}
                    }
                }
            ]
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_find_toolchain_exact() {
        let m = sample_manifest();
        let entry = find_toolchain(&m, "nxp", "14.2.1").unwrap();
        assert_eq!(entry.gcc, "14.2.1");
    }

    #[test]
    fn test_find_toolchain_prefix() {
        let m = sample_manifest();
        let entry = find_toolchain(&m, "nxp", "14.2").unwrap();
        assert_eq!(entry.version, "14.2.1");
    }

    #[test]
    fn test_find_toolchain_latest() {
        let m = sample_manifest();
        let entry = find_toolchain(&m, "stm", "latest").unwrap();
        assert_eq!(entry.version, "13.3.1");
    }

    #[test]
    fn test_find_toolchain_missing() {
        let m = sample_manifest();
        assert!(find_toolchain(&m, "nxp", "99.0").is_err());
    }

    #[test]
    fn test_resolve_asset() {
        let m = sample_manifest();
        let entry = find_toolchain(&m, "nxp", "14.2.1").unwrap();
        let asset = resolve_asset(entry).unwrap();
        assert!(asset.file.contains("linux") || asset.file.contains("win"));
    }

    #[test]
    fn test_available_versions() {
        let m = sample_manifest();
        let versions = available_versions(&m);
        assert_eq!(versions.len(), 2);
    }
}
