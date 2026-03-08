use anyhow::{bail, Result};
use colored::*;

use crate::utils::paths;

/// Parse "vendor:version" spec, e.g. "nxp:14.2" or "nxp:14.2.1"
pub fn parse_spec(spec: &str) -> Result<(String, String)> {
    if let Some((vendor, version)) = spec.split_once(':') {
        Ok((vendor.to_string(), version.to_string()))
    } else {
        bail!(
            "Invalid toolchain spec '{}'. Use 'vendor:version' format (e.g., nxp:14.2, stm:13.3)",
            spec
        );
    }
}

pub fn install(spec: &str, force: bool) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;
    println!(
        "{} Installing {} ARM GCC {}...",
        "📦".bold(),
        vendor.to_uppercase().cyan(),
        version.cyan()
    );
    // TODO: Implement via toolchain_manager
    println!("   {}", "(Not yet implemented)".dimmed());
    Ok(())
}

pub fn list(available: bool) -> Result<()> {
    let tc_dir = paths::toolchains_dir()?;

    println!("{}", "Installed toolchains:".bold());

    if !tc_dir.exists() {
        println!("   {}", "(none)".dimmed());
        return Ok(());
    }

    let mut found = false;
    let mut entries: Vec<_> = std::fs::read_dir(&tc_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Verify it has bin/arm-none-eabi-gcc
        let gcc = entry.path().join("bin").join("arm-none-eabi-gcc");
        let gcc_exe = entry.path().join("bin").join("arm-none-eabi-gcc.exe");
        if gcc.exists() || gcc_exe.exists() {
            found = true;
            // TODO: Check active version from config
            println!("    {}", name_str.green());
        }
    }

    if !found {
        println!("   {}", "(none)".dimmed());
    }

    if available {
        println!();
        println!("{}", "Available toolchains (remote):".bold());
        // TODO: Fetch versions.json and display
        println!("   {}", "(Not yet implemented)".dimmed());
    }

    Ok(())
}

pub fn use_version(spec: &str) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;
    let tc_path = paths::toolchain_path(&vendor, &version)?;

    if !tc_path.exists() {
        bail!(
            "Toolchain {}-{} is not installed. Run 'embtool toolchain install {}' first.",
            vendor,
            version,
            spec
        );
    }

    // Update config
    let mut config = crate::core::config::load()?;
    config.toolchain.default = Some(format!("{}-{}", vendor, version));
    crate::core::config::save(&config)?;

    println!(
        "{} Switched to {} ARM GCC {}",
        "🔄".bold(),
        vendor.to_uppercase().cyan(),
        version.cyan()
    );

    Ok(())
}

pub fn remove(spec: &str) -> Result<()> {
    let (vendor, version) = parse_spec(spec)?;
    let tc_path = paths::toolchain_path(&vendor, &version)?;

    if !tc_path.exists() {
        bail!("Toolchain {}-{} is not installed.", vendor, version);
    }

    std::fs::remove_dir_all(&tc_path)?;
    println!(
        "{} Removed {} ARM GCC {}",
        "🗑️ ".bold(),
        vendor.to_uppercase(),
        version
    );

    // Clear default if it was the active one
    let mut config = crate::core::config::load()?;
    let id = format!("{}-{}", vendor, version);
    if config.toolchain.default.as_deref() == Some(&id) {
        config.toolchain.default = None;
        crate::core::config::save(&config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec() {
        let (v, ver) = parse_spec("nxp:14.2.1").unwrap();
        assert_eq!(v, "nxp");
        assert_eq!(ver, "14.2.1");

        let (v, ver) = parse_spec("stm:13.3").unwrap();
        assert_eq!(v, "stm");
        assert_eq!(ver, "13.3");
    }

    #[test]
    fn test_parse_spec_invalid() {
        assert!(parse_spec("14.2.1").is_err());
        assert!(parse_spec("").is_err());
    }
}
