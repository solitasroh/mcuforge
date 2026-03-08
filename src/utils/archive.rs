use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Check if 7z is available
pub fn check_7z() -> Result<PathBuf> {
    // Try common names
    for name in &["7z", "7zz", "7za"] {
        if let Ok(output) = std::process::Command::new(name)
            .arg("--help")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            if output.success() || output.code() == Some(0) {
                return Ok(PathBuf::from(name));
            }
        }
    }

    // Windows: check common paths
    if cfg!(windows) {
        let paths = [
            r"C:\Program Files\7-Zip\7z.exe",
            r"C:\Program Files (x86)\7-Zip\7z.exe",
        ];
        for p in &paths {
            if Path::new(p).exists() {
                return Ok(PathBuf::from(p));
            }
        }
    }

    bail!(
        "7z not found. Please install:\n  \
         Linux: sudo apt install p7zip-full\n  \
         Windows: https://www.7-zip.org/download.html"
    );
}

/// Extract a 7z archive to a destination directory
pub fn extract(
    archive_path: &Path,
    dest_dir: &Path,
    toolchain_name: &str,
) -> Result<PathBuf> {
    let seven_z = check_7z()?;
    let target = dest_dir.join(toolchain_name);

    // Clean up if exists
    if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }

    // Extract to temp dir first
    let tmp_dir = dest_dir.join(format!(".{}-extracting", toolchain_name));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    std::fs::create_dir_all(&tmp_dir)?;

    let status = std::process::Command::new(&seven_z)
        .arg("x")
        .arg(archive_path)
        .arg(format!("-o{}", tmp_dir.display()))
        .arg("-y")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("Failed to run 7z on {}", archive_path.display()))?;

    if !status.success() {
        std::fs::remove_dir_all(&tmp_dir)?;
        bail!("7z extraction failed with exit code {:?}", status.code());
    }

    // Move to final location
    std::fs::rename(&tmp_dir, &target)
        .with_context(|| format!("Failed to move extracted files to {}", target.display()))?;

    // Verify: bin/arm-none-eabi-gcc must exist
    let gcc_name = if cfg!(windows) {
        "arm-none-eabi-gcc.exe"
    } else {
        "arm-none-eabi-gcc"
    };

    if !target.join("bin").join(gcc_name).exists() {
        bail!(
            "Extraction completed but bin/{} not found in {}",
            gcc_name,
            target.display()
        );
    }

    Ok(target)
}
