use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::core::tool_provider::{ArchiveType, ToolProvider};
use crate::utils::paths;

/// Supported clang major versions (from cpp-linter/clang-tools-static-binaries)
const KNOWN_VERSIONS: &[&str] = &[
    "22", "21", "20", "19", "18", "17", "16", "15", "14", "13", "12", "11",
];

const GITHUB_REPO: &str = "cpp-linter/clang-tools-static-binaries";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClangTool {
    Format,
    Tidy,
}

impl ClangTool {
    pub fn name(&self) -> &'static str {
        match self {
            ClangTool::Format => "clang-format",
            ClangTool::Tidy => "clang-tidy",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "clang-format" => Ok(ClangTool::Format),
            "clang-tidy" => Ok(ClangTool::Tidy),
            _ => bail!("Unknown clang tool '{}'. Use: clang-format, clang-tidy", s),
        }
    }
}

pub struct ClangProvider {
    pub tool: ClangTool,
}

impl ClangProvider {
    pub fn new(tool: ClangTool) -> Self {
        Self { tool }
    }

    fn platform_suffix() -> Result<&'static str> {
        let os = super::tool_provider::platform_os();
        let arch = super::tool_provider::platform_arch();

        match (os, arch) {
            ("linux", "x64") => Ok("linux-amd64"),
            ("windows", "x64") => Ok("windows-amd64"),
            ("macos", "x64") => Ok("macos-intel-amd64"),
            ("macos", "arm64") => Ok("macosx-arm64"),
            _ => bail!("Unsupported platform for clang tools: {}-{}", os, arch),
        }
    }

    fn binary_name(&self, version: &str) -> Result<String> {
        let suffix = Self::platform_suffix()?;
        let ext = if cfg!(windows) { ".exe" } else { "" };
        Ok(format!("{}-{}_{}{}", self.tool.name(), version, suffix, ext))
    }

    fn tools_dir(&self) -> Result<PathBuf> {
        let home = paths::embtool_home()?;
        Ok(home.join("tools").join(self.tool.name()))
    }

    /// Get release tag — static binaries repo uses a rolling tag
    fn release_tag() -> Result<String> {
        // The repo uses "master-{hash}" tags. We use the "latest" redirect.
        Ok("latest".to_string())
    }
}

impl ToolProvider for ClangProvider {
    fn name(&self) -> &str {
        self.tool.name()
    }

    fn download_url(&self, version: &str) -> Result<String> {
        let filename = self.binary_name(version)?;
        // Use GitHub latest release redirect
        Ok(format!(
            "https://github.com/{}/releases/latest/download/{}",
            GITHUB_REPO, filename
        ))
    }

    fn archive_type(&self) -> ArchiveType {
        ArchiveType::SingleBinary
    }

    fn archive_filename(&self, version: &str) -> Result<String> {
        self.binary_name(version)
    }

    fn install_dir(&self, version: &str) -> Result<PathBuf> {
        Ok(self.tools_dir()?.join(version))
    }

    fn verify_install(&self, install_path: &Path) -> Result<String> {
        let bin_name = if cfg!(windows) {
            format!("{}.exe", self.tool.name())
        } else {
            self.tool.name().to_string()
        };
        let bin = install_path.join(&bin_name);
        if !bin.exists() {
            bail!("{} not found at {}", bin_name, install_path.display());
        }

        let output = std::process::Command::new(&bin)
            .arg("--version")
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        // clang-format: "clang-format version 18.1.8 (...)"
        // clang-tidy: "LLVM version 18.1.8"
        let version = stdout
            .lines()
            .find_map(|line| {
                if let Some(rest) = line.strip_prefix("clang-format version ") {
                    Some(rest.split_whitespace().next().unwrap_or("?").to_string())
                } else if let Some(rest) = line.strip_prefix("LLVM version ") {
                    Some(rest.trim().to_string())
                } else if line.contains("version ") {
                    line.split("version ")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "?".to_string());

        Ok(version)
    }

    fn checksum_url(&self, version: &str) -> Result<Option<String>> {
        let filename = self.binary_name(version)?;
        Ok(Some(format!(
            "https://github.com/{}/releases/latest/download/{}.sha512sum",
            GITHUB_REPO, filename
        )))
    }

    fn available_versions(&self) -> Result<Vec<String>> {
        Ok(KNOWN_VERSIONS.iter().map(|s| s.to_string()).collect())
    }
}

/// Install a single binary tool (clang-format or clang-tidy)
pub fn install_binary(url: &str, install_dir: &Path, binary_name: &str) -> Result<PathBuf> {
    std::fs::create_dir_all(install_dir)?;

    let dest = install_dir.join(binary_name);

    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        bail!("Download failed: HTTP {} from {}", response.status(), url);
    }

    let bytes = response.bytes()?;
    std::fs::write(&dest, &bytes)?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(dest)
}

/// List installed versions for a tool
pub fn list_installed(tool: ClangTool) -> Result<Vec<(String, PathBuf)>> {
    let provider = ClangProvider::new(tool);
    let tools_dir = provider.tools_dir()?;
    if !tools_dir.exists() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&tools_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        result.push((name, entry.path()));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_format() {
        let p = ClangProvider::new(ClangTool::Format);
        let url = p.download_url("18").unwrap();
        assert!(url.contains("clang-format-18"));
        assert!(url.contains("cpp-linter"));
    }

    #[test]
    fn test_download_url_tidy() {
        let p = ClangProvider::new(ClangTool::Tidy);
        let url = p.download_url("22").unwrap();
        assert!(url.contains("clang-tidy-22"));
    }

    #[test]
    fn test_available_versions() {
        let p = ClangProvider::new(ClangTool::Format);
        let versions = p.available_versions().unwrap();
        assert!(versions.contains(&"18".to_string()));
        assert!(versions.contains(&"22".to_string()));
        assert_eq!(versions.len(), 12);
    }
}
