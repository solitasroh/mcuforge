use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

const PROJECT_FILE: &str = "embtool.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    pub target: TargetConfig,
    pub toolchain: ProjectToolchain,
    #[serde(default)]
    pub cmake: CmakeConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub debug: ProjectDebug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmakeConfig {
    #[serde(default = "default_cmake_version")]
    pub version: String,
}

fn default_cmake_version() -> String {
    "3.28".to_string()
}

impl Default for CmakeConfig {
    fn default() -> Self {
        Self {
            version: default_cmake_version(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    #[serde(default, rename = "clang-format")]
    pub clang_format: Option<ToolVersionConfig>,
    #[serde(default, rename = "clang-tidy")]
    pub clang_tidy: Option<ToolVersionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersionConfig {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default = "default_project_version")]
    pub version: String,
    #[serde(default = "default_project_type", rename = "type")]
    pub project_type: String,
}

fn default_project_type() -> String {
    "application".to_string()
}

fn default_project_version() -> String {
    "0.1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub mcu: String,
    pub core: String,
    #[serde(default = "default_fpu")]
    pub fpu: String,
    #[serde(default)]
    pub flash: String,
    #[serde(default)]
    pub ram: String,
}

fn default_fpu() -> String {
    "soft".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectToolchain {
    /// Vendor: "nxp" or "stm"
    pub vendor: String,
    /// Version: "14.2.1"
    pub version: String,
}

impl ProjectToolchain {
    /// Returns the toolchain ID in "vendor-version" format
    pub fn id(&self) -> String {
        format!("{}-{}", self.vendor, self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_c_standard")]
    pub c_standard: String,
    #[serde(default)]
    pub optimization: OptimizationConfig,
    #[serde(default)]
    pub linker_script: String,
    #[serde(default)]
    pub defines: DefinesConfig,
    #[serde(default)]
    pub flags: FlagsConfig,
}

fn default_c_standard() -> String {
    "c99".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub debug: String,
    pub release: String,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            debug: "O0".to_string(),
            release: "O1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefinesConfig {
    #[serde(default)]
    pub target: Vec<String>,
    #[serde(default)]
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlagsConfig {
    #[serde(default)]
    pub common: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDebug {
    #[serde(default = "default_probe")]
    pub probe: String,
    #[serde(default = "default_interface")]
    pub interface: String,
}

fn default_probe() -> String {
    "pemicro".to_string()
}

fn default_interface() -> String {
    "swd".to_string()
}

impl Default for ProjectDebug {
    fn default() -> Self {
        Self {
            probe: default_probe(),
            interface: default_interface(),
        }
    }
}

/// Search for embtool.toml from current directory up to root
pub fn find_project() -> Result<PathBuf> {
    find_project_from(&std::env::current_dir()?)
}

/// Search for embtool.toml starting from a specific directory
pub fn find_project_from(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join(PROJECT_FILE);
        if candidate.exists() {
            return Ok(candidate);
        }
        if !dir.pop() {
            bail!(
                "embtool.toml not found. Run 'embtool new <name> --mcu <mcu>' to create a project."
            );
        }
    }
}

/// Load and parse embtool.toml
pub fn load(path: &Path) -> Result<ProjectConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let config: ProjectConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}

/// Save project config to embtool.toml
#[allow(dead_code)]
pub fn save(path: &Path, config: &ProjectConfig) -> Result<()> {
    let content = toml::to_string_pretty(config).context("Failed to serialize project config")?;
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_config() {
        let toml_str = r#"
[project]
name = "a2750lm_application"
version = "1.0.0"

[target]
mcu = "MK64FN1M0VLL12"
core = "cortex-m4"
fpu = "soft"
flash = "1M"
ram = "256K"

[toolchain]
vendor = "nxp"
version = "14.2.1"

[build]
c_standard = "c99"
linker_script = "system/linkerscript.ld"

[build.optimization]
debug = "O0"
release = "O1"

[build.defines]
target = ["MK64F12"]
custom = []

[build.flags]
common = ["-ffunction-sections", "-fno-common"]

[debug]
probe = "pemicro"
interface = "swd"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "a2750lm_application");
        assert_eq!(config.target.mcu, "MK64FN1M0VLL12");
        assert_eq!(config.target.core, "cortex-m4");
        assert_eq!(config.toolchain.vendor, "nxp");
        assert_eq!(config.toolchain.version, "14.2.1");
        assert_eq!(config.toolchain.id(), "nxp-14.2.1");
        assert_eq!(config.build.defines.target, vec!["MK64F12"]);
    }

    #[test]
    fn test_toolchain_id() {
        let tc = ProjectToolchain {
            vendor: "stm".to_string(),
            version: "13.3.1".to_string(),
        };
        assert_eq!(tc.id(), "stm-13.3.1");
    }

    #[test]
    fn test_find_project_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let result = find_project_from(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_find_project_in_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub").join("deep");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(tmp.path().join("embtool.toml"), "[project]\nname=\"test\"\nversion=\"0.1\"\n[target]\nmcu=\"K64\"\ncore=\"cortex-m4\"\n[toolchain]\nvendor=\"nxp\"\nversion=\"14.2.1\"").unwrap();
        let found = find_project_from(&sub).unwrap();
        assert_eq!(found, tmp.path().join("embtool.toml"));
    }

    #[test]
    fn test_save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("embtool.toml");

        let config = ProjectConfig {
            project: ProjectMeta {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                project_type: "application".to_string(),
            },
            target: TargetConfig {
                mcu: "MK64FN1M0VLL12".to_string(),
                core: "cortex-m4".to_string(),
                fpu: "soft".to_string(),
                flash: "1M".to_string(),
                ram: "256K".to_string(),
            },
            toolchain: ProjectToolchain {
                vendor: "nxp".to_string(),
                version: "14.2.1".to_string(),
            },
            cmake: CmakeConfig::default(),
            tools: ToolsConfig::default(),
            build: BuildConfig::default(),
            debug: ProjectDebug::default(),
        };

        save(&path, &config).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.project.name, "test");
        assert_eq!(loaded.toolchain.id(), "nxp-14.2.1");
    }
}
