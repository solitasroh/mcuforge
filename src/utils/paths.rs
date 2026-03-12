use std::path::PathBuf;

use anyhow::{Context, Result};

/// embtool 홈 디렉토리 반환
/// - 환경변수 EMBTOOL_HOME 오버라이드 지원
/// - Win: %USERPROFILE%/.embtool
/// - Linux: $HOME/.embtool
pub fn embtool_home() -> Result<PathBuf> {
    if let Ok(custom) = std::env::var("EMBTOOL_HOME") {
        return Ok(PathBuf::from(custom));
    }
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".embtool"))
}

/// ~/.embtool/toolchains/
pub fn toolchains_dir() -> Result<PathBuf> {
    Ok(embtool_home()?.join("toolchains"))
}

/// ~/.embtool/cache/
pub fn cache_dir() -> Result<PathBuf> {
    Ok(embtool_home()?.join("cache"))
}

/// ~/.embtool/config.toml
pub fn global_config_path() -> Result<PathBuf> {
    Ok(embtool_home()?.join("config.toml"))
}

/// 디렉토리가 없으면 생성 (toolchains, cache 포함)
pub fn ensure_dirs() -> Result<()> {
    let dirs = [embtool_home()?, toolchains_dir()?, cache_dir()?];
    for dir in &dirs {
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
        }
    }
    Ok(())
}

/// 특정 벤더-버전 툴체인의 설치 경로
/// 예: ~/.embtool/toolchains/nxp-14.2.1/
pub fn toolchain_path(vendor: &str, version: &str) -> Result<PathBuf> {
    Ok(toolchains_dir()?.join(format!("{}-{}", vendor, version)))
}

/// 특정 벤더-버전 툴체인의 bin/ 경로
#[allow(dead_code)]
pub fn toolchain_bin_path(vendor: &str, version: &str) -> Result<PathBuf> {
    Ok(toolchain_path(vendor, version)?.join("bin"))
}

/// ~/.embtool/claude-skills/
pub fn skills_cache_dir() -> Result<PathBuf> {
    Ok(embtool_home()?.join("claude-skills"))
}

/// 캐시된 versions.json 경로
pub fn cached_versions_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join("versions.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embtool_home_default() {
        // SAFETY: test-only, single-threaded
        unsafe { std::env::remove_var("EMBTOOL_HOME") };
        let home = embtool_home().unwrap();
        assert!(home.ends_with(".embtool"));
    }

    #[test]
    fn test_embtool_home_override() {
        unsafe { std::env::set_var("EMBTOOL_HOME", "/tmp/test-embtool") };
        let home = embtool_home().unwrap();
        assert_eq!(home, PathBuf::from("/tmp/test-embtool"));
        unsafe { std::env::remove_var("EMBTOOL_HOME") };
    }

    #[test]
    fn test_subdirectories() {
        unsafe { std::env::set_var("EMBTOOL_HOME", "/tmp/test-embtool") };
        assert_eq!(toolchains_dir().unwrap(), PathBuf::from("/tmp/test-embtool/toolchains"));
        assert_eq!(cache_dir().unwrap(), PathBuf::from("/tmp/test-embtool/cache"));
        assert_eq!(global_config_path().unwrap(), PathBuf::from("/tmp/test-embtool/config.toml"));
        unsafe { std::env::remove_var("EMBTOOL_HOME") };
    }

    #[test]
    fn test_toolchain_path() {
        unsafe { std::env::set_var("EMBTOOL_HOME", "/tmp/test-embtool") };
        assert_eq!(
            toolchain_path("nxp", "14.2.1").unwrap(),
            PathBuf::from("/tmp/test-embtool/toolchains/nxp-14.2.1")
        );
        assert_eq!(
            toolchain_bin_path("stm", "13.3.1").unwrap(),
            PathBuf::from("/tmp/test-embtool/toolchains/stm-13.3.1/bin")
        );
        unsafe { std::env::remove_var("EMBTOOL_HOME") };
    }

    #[test]
    fn test_ensure_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("EMBTOOL_HOME", tmp.path()) };
        ensure_dirs().unwrap();
        assert!(tmp.path().join("toolchains").exists());
        assert!(tmp.path().join("cache").exists());
        unsafe { std::env::remove_var("EMBTOOL_HOME") };
    }
}
