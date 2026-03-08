use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::core::tool_provider::{ArchiveType, ToolProvider};
use crate::utils::paths;

/// Known stable CMake versions (embedded, no API call needed)
const KNOWN_VERSIONS: &[&str] = &[
    "4.0.2", "4.0.1", "4.0.0",
    "3.31.5", "3.31.4", "3.31.3", "3.31.2", "3.31.1", "3.31.0",
    "3.30.8", "3.30.7", "3.30.6", "3.30.5", "3.30.4", "3.30.3", "3.30.2", "3.30.1", "3.30.0",
    "3.29.8", "3.29.7", "3.29.6", "3.29.5", "3.29.4", "3.29.3", "3.29.2", "3.29.1", "3.29.0",
    "3.28.6", "3.28.5", "3.28.4", "3.28.3", "3.28.2", "3.28.1", "3.28.0",
    "3.27.9", "3.27.8", "3.27.7", "3.27.6", "3.27.5", "3.27.4", "3.27.3", "3.27.2", "3.27.1", "3.27.0",
    "3.26.6", "3.26.5", "3.26.4", "3.26.3", "3.26.2", "3.26.1", "3.26.0",
    "3.25.3", "3.25.2", "3.25.1", "3.25.0",
    "3.24.4", "3.24.3", "3.24.2", "3.24.1", "3.24.0",
    "3.23.5", "3.23.4", "3.23.3", "3.23.2", "3.23.1", "3.23.0",
    "3.22.6", "3.22.5", "3.22.4", "3.22.3", "3.22.2", "3.22.1", "3.22.0",
    "3.21.7", "3.21.6", "3.21.5", "3.21.4", "3.21.3", "3.21.2", "3.21.1", "3.21.0",
    "3.20.6", "3.20.5", "3.20.4", "3.20.3", "3.20.2", "3.20.1", "3.20.0",
    "3.19.8", "3.19.7", "3.19.6", "3.19.5", "3.19.4", "3.19.3", "3.19.2", "3.19.1", "3.19.0",
    "3.18.6", "3.18.5", "3.18.4", "3.18.3", "3.18.2", "3.18.1", "3.18.0",
    "3.17.5", "3.17.4", "3.17.3", "3.17.2", "3.17.1", "3.17.0",
    "3.16.9", "3.16.8", "3.16.7", "3.16.6", "3.16.5", "3.16.4", "3.16.3", "3.16.2", "3.16.1", "3.16.0",
];

pub const DEFAULT_CMAKE_VERSION: &str = "3.28";

pub struct CmakeProvider;

impl CmakeProvider {
    /// Resolve "3.28" → "3.28.6" (latest patch)
    pub fn resolve_version(spec: &str) -> Result<String> {
        if spec == "latest" {
            return Ok(KNOWN_VERSIONS[0].to_string());
        }

        // Exact match
        if KNOWN_VERSIONS.contains(&spec) {
            return Ok(spec.to_string());
        }

        // Prefix match: "3.28" → "3.28.6" (first match = latest patch)
        for v in KNOWN_VERSIONS {
            if v.starts_with(spec) {
                return Ok(v.to_string());
            }
        }

        bail!("CMake version '{}' not found. Use 'embtool cmake list --available' to see options.", spec);
    }

    fn platform_filename(version: &str) -> Result<String> {
        let os = super::tool_provider::platform_os();
        let arch = super::tool_provider::platform_arch();

        match (os, arch) {
            ("linux", "x64") => Ok(format!("cmake-{}-linux-x86_64.tar.gz", version)),
            ("linux", "arm64") => Ok(format!("cmake-{}-linux-aarch64.tar.gz", version)),
            ("windows", "x64") => Ok(format!("cmake-{}-windows-x86_64.zip", version)),
            ("macos", _) => Ok(format!("cmake-{}-macos-universal.tar.gz", version)),
            _ => bail!("Unsupported platform: {}-{}", os, arch),
        }
    }

    fn cmake_dir() -> Result<PathBuf> {
        let home = paths::embtool_home()?;
        Ok(home.join("cmake"))
    }
}

impl ToolProvider for CmakeProvider {
    fn name(&self) -> &str {
        "cmake"
    }

    fn download_url(&self, version: &str) -> Result<String> {
        let filename = Self::platform_filename(version)?;
        Ok(format!(
            "https://github.com/Kitware/CMake/releases/download/v{}/{}",
            version, filename
        ))
    }

    fn archive_type(&self) -> ArchiveType {
        if cfg!(target_os = "windows") {
            ArchiveType::Zip
        } else {
            ArchiveType::TarGz
        }
    }

    fn archive_filename(&self, version: &str) -> Result<String> {
        Self::platform_filename(version)
    }

    fn install_dir(&self, version: &str) -> Result<PathBuf> {
        Ok(Self::cmake_dir()?.join(version))
    }

    fn verify_install(&self, install_path: &Path) -> Result<String> {
        let cmake_bin = if cfg!(windows) { "cmake.exe" } else { "cmake" };
        let cmake = install_path.join("bin").join(cmake_bin);
        if !cmake.exists() {
            bail!("cmake binary not found at {}", cmake.display());
        }

        let output = std::process::Command::new(&cmake)
            .arg("--version")
            .output()
            .context("Failed to run cmake")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // "cmake version 3.28.6"
        stdout
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("cmake version "))
            .map(|v| v.trim().to_string())
            .ok_or_else(|| anyhow::anyhow!("Could not parse cmake version"))
    }

    fn checksum_url(&self, _version: &str) -> Result<Option<String>> {
        // GitHub provides SHA256 in a separate file but format varies
        // We rely on HTTPS + known URL for now
        Ok(None)
    }

    fn available_versions(&self) -> Result<Vec<String>> {
        Ok(KNOWN_VERSIONS.iter().map(|s| s.to_string()).collect())
    }
}

/// Extract CMake tar.gz archive
pub fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);

    // CMake tar.gz has a top-level dir like "cmake-3.28.6-linux-x86_64/"
    // We need to strip it and extract contents directly to dest
    let tmp = dest.parent().unwrap().join(format!(".{}-extracting", dest.file_name().unwrap().to_string_lossy()));
    if tmp.exists() { std::fs::remove_dir_all(&tmp)?; }
    std::fs::create_dir_all(&tmp)?;

    tar.unpack(&tmp)?;

    // Find the top-level directory
    let entries: Vec<_> = std::fs::read_dir(&tmp)?.filter_map(|e| e.ok()).collect();
    if entries.len() == 1 && entries[0].path().is_dir() {
        // Single top-level dir — move it to dest
        let top = entries[0].path();
        if dest.exists() { std::fs::remove_dir_all(dest)?; }
        std::fs::rename(&top, dest)?;
        std::fs::remove_dir_all(&tmp)?;
    } else {
        // No wrapper dir — rename tmp to dest
        if dest.exists() { std::fs::remove_dir_all(dest)?; }
        std::fs::rename(&tmp, dest)?;
    }

    Ok(())
}

/// Extract CMake zip archive (Windows)
pub fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;

    let tmp = dest.parent().unwrap().join(format!(".{}-extracting", dest.file_name().unwrap().to_string_lossy()));
    if tmp.exists() { std::fs::remove_dir_all(&tmp)?; }
    std::fs::create_dir_all(&tmp)?;

    zip.extract(&tmp)?;

    // Same logic: find top-level dir
    let entries: Vec<_> = std::fs::read_dir(&tmp)?.filter_map(|e| e.ok()).collect();
    if entries.len() == 1 && entries[0].path().is_dir() {
        let top = entries[0].path();
        if dest.exists() { std::fs::remove_dir_all(dest)?; }
        std::fs::rename(&top, dest)?;
        std::fs::remove_dir_all(&tmp)?;
    } else {
        if dest.exists() { std::fs::remove_dir_all(dest)?; }
        std::fs::rename(&tmp, dest)?;
    }

    Ok(())
}

/// List installed CMake versions
pub fn list_installed() -> Result<Vec<(String, PathBuf)>> {
    let cmake_dir = CmakeProvider::cmake_dir()?;
    if !cmake_dir.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&cmake_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && !e.file_name().to_string_lossy().starts_with('.'))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        result.push((name, path));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_exact() {
        let v = CmakeProvider::resolve_version("3.28.6").unwrap();
        assert_eq!(v, "3.28.6");
    }

    #[test]
    fn test_resolve_prefix() {
        let v = CmakeProvider::resolve_version("3.28").unwrap();
        assert_eq!(v, "3.28.6");
    }

    #[test]
    fn test_resolve_latest() {
        let v = CmakeProvider::resolve_version("latest").unwrap();
        assert_eq!(v, "4.0.2");
    }

    #[test]
    fn test_resolve_missing() {
        assert!(CmakeProvider::resolve_version("2.8").is_err());
    }

    #[test]
    fn test_download_url() {
        let provider = CmakeProvider;
        let url = provider.download_url("3.28.6").unwrap();
        assert!(url.contains("github.com/Kitware/CMake"));
        assert!(url.contains("3.28.6"));
    }
}
