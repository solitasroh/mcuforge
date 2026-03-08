use std::path::{Path, PathBuf};
use anyhow::Result;

/// Archive type for extraction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchiveType {
    TarGz,
    Zip,
    SingleBinary,
}

/// Result of a tool installation
pub struct InstallResult {
    #[allow(dead_code)]
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub size_mb: u64,
    pub already_installed: bool,
}

/// Platform detection
pub fn platform_os() -> &'static str {
    if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "linux") { "linux" }
    else if cfg!(target_os = "macos") { "macos" }
    else { "unknown" }
}

pub fn platform_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") { "x64" }
    else if cfg!(target_arch = "aarch64") { "arm64" }
    else { "unknown" }
}

/// Tool provider trait — each tool type implements this
pub trait ToolProvider {
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Build download URL for a specific version
    fn download_url(&self, version: &str) -> Result<String>;

    /// What kind of archive is it?
    fn archive_type(&self) -> ArchiveType;

    /// Expected file name after download
    fn archive_filename(&self, version: &str) -> Result<String>;

    /// Installation directory
    fn install_dir(&self, version: &str) -> Result<PathBuf>;

    /// Check if a version is already installed, return version string
    fn verify_install(&self, install_path: &Path) -> Result<String>;

    /// SHA verification URL (if available)
    #[allow(dead_code)]
    fn checksum_url(&self, version: &str) -> Result<Option<String>>;

    /// List available versions
    fn available_versions(&self) -> Result<Vec<String>>;
}

// Re-export from utils::fs for backward compat
pub use crate::utils::fs::{dir_size_mb, file_size_mb};
