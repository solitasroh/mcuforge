use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};

use crate::core::config;

/// Download a file from URL with progress bar and SHA256 verification
pub fn download_file(
    url: &str,
    dest_dir: &Path,
    filename: &str,
    expected_sha256: &str,
    show_progress: bool,
) -> Result<PathBuf> {
    let dest = dest_dir.join(filename);

    // Check cache: file exists + SHA256 matches
    if dest.exists() {
        if verify_sha256(&dest, expected_sha256)? {
            return Ok(dest);
        }
        // Hash mismatch — re-download
        std::fs::remove_file(&dest)?;
    }

    // Ensure directory exists
    std::fs::create_dir_all(dest_dir)?;

    // Download
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("Failed to connect to {}", url))?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {} from {}", response.status(), url);
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = if show_progress && !config::is_ci() && total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("   {bar:40.cyan/dim} {percent}% ({bytes}/{total_bytes})")
                .unwrap()
                .progress_chars("━╸─"),
        );
        Some(pb)
    } else {
        if config::is_ci() && total_size > 0 {
            eprintln!("   Downloading {} ({:.1} MB)...", filename, total_size as f64 / 1_048_576.0);
        }
        None
    };

    let mut file = std::fs::File::create(&dest)
        .with_context(|| format!("Failed to create {}", dest.display()))?;

    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut reader = response;

    loop {
        let mut buf = [0u8; 8192];
        let n = std::io::Read::read(&mut reader, &mut buf)
            .context("Download interrupted")?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        if let Some(ref pb) = pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Verify SHA256
    let hash = format!("{:x}", hasher.finalize());
    if hash != expected_sha256 {
        std::fs::remove_file(&dest)?;
        bail!(
            "SHA256 mismatch for {}:\n  expected: {}\n  got:      {}",
            filename, expected_sha256, hash
        );
    }

    Ok(dest)
}

/// Verify SHA256 hash of a file
pub fn verify_sha256(path: &Path, expected: &str) -> Result<bool> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = format!("{:x}", hasher.finalize());
    Ok(hash == expected)
}

/// Get cache directory size in MB
pub fn cache_size_mb() -> Result<u64> {
    let cache = crate::utils::paths::cache_dir()?;
    Ok(dir_size(&cache) / (1024 * 1024))
}

/// Clear download cache
pub fn clear_cache() -> Result<()> {
    let cache = crate::utils::paths::cache_dir()?;
    if cache.exists() {
        std::fs::remove_dir_all(&cache)?;
        std::fs::create_dir_all(&cache)?;
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
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
