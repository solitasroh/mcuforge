use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::utils::paths;

const DEFAULT_REGISTRY_URL: &str = "https://pub-25d9755030a54c3280b7a9f68e9bf67c.r2.dev";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub toolchain: ToolchainConfig,
    #[serde(default)]
    pub registry: RegistryConfig,
    #[serde(default)]
    pub mirror: MirrorConfig,
    #[serde(default)]
    pub debug: DebugConfig,
    #[serde(default)]
    pub ci: CiConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            toolchain: ToolchainConfig::default(),
            registry: RegistryConfig::default(),
            mirror: MirrorConfig::default(),
            debug: DebugConfig::default(),
            ci: CiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolchainConfig {
    /// Active toolchain, e.g. "nxp-14.2.1"
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Base URL for toolchain downloads and versions.json
    pub url: String,
    /// Cache TTL for versions.json (hours)
    pub cache_ttl_hours: u32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_REGISTRY_URL.to_string(),
            cache_ttl_hours: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Enable NAS/local mirror
    pub enabled: bool,
    /// Mirror type: "local" or "http"
    #[serde(default = "default_mirror_type")]
    pub mirror_type: String,
    /// Mirror URL or path
    pub url: String,
    /// Fall back to R2 if mirror fails
    pub fallback: bool,
}

fn default_mirror_type() -> String {
    "local".to_string()
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mirror_type: "local".to_string(),
            url: String::new(),
            fallback: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub default_probe: String,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            default_probe: "pemicro".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiConfig {
    pub auto_detect: bool,
}

impl Default for CiConfig {
    fn default() -> Self {
        Self { auto_detect: true }
    }
}

/// Load global config from ~/.embtool/config.toml
/// Returns default config if file doesn't exist
pub fn load() -> Result<GlobalConfig> {
    let path = paths::global_config_path()?;
    load_from(&path)
}

/// Load config from a specific path
pub fn load_from(path: &Path) -> Result<GlobalConfig> {
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let config: GlobalConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    Ok(config)
}

/// Save global config to ~/.embtool/config.toml
pub fn save(config: &GlobalConfig) -> Result<()> {
    let path = paths::global_config_path()?;
    save_to(&path, config)
}

/// Save config to a specific path
pub fn save_to(path: &Path, config: &GlobalConfig) -> Result<()> {
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write config: {}", path.display()))?;
    Ok(())
}

/// Detect if running in CI environment
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("JENKINS_URL").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GlobalConfig::default();
        assert_eq!(config.registry.url, DEFAULT_REGISTRY_URL);
        assert_eq!(config.registry.cache_ttl_hours, 24);
        assert!(!config.mirror.enabled);
        assert!(config.mirror.fallback);
        assert!(config.toolchain.default.is_none());
    }

    #[test]
    fn test_load_missing_file() {
        let config = load_from(Path::new("/tmp/nonexistent-embtool-config.toml")).unwrap();
        assert_eq!(config.registry.url, DEFAULT_REGISTRY_URL);
    }

    #[test]
    fn test_save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");

        let mut config = GlobalConfig::default();
        config.toolchain.default = Some("nxp-14.2.1".to_string());
        config.mirror.enabled = true;
        config.mirror.url = "\\\\nas\\share\\toolchains".to_string();

        save_to(&path, &config).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.toolchain.default, Some("nxp-14.2.1".to_string()));
        assert!(loaded.mirror.enabled);
        assert_eq!(loaded.mirror.url, "\\\\nas\\share\\toolchains");
    }

    #[test]
    fn test_parse_partial_config() {
        let toml_str = r#"
[toolchain]
default = "stm-13.3.1"
"#;
        let config: GlobalConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.toolchain.default, Some("stm-13.3.1".to_string()));
        // Other fields should have defaults
        assert_eq!(config.registry.url, DEFAULT_REGISTRY_URL);
        assert!(!config.mirror.enabled);
    }
}
