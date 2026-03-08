use std::path::Path;

/// Get directory size in bytes (recursive)
pub fn dir_size(path: &Path) -> u64 {
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

/// Get directory size in MB
pub fn dir_size_mb(path: &Path) -> u64 {
    dir_size(path) / (1024 * 1024)
}

/// Get file size in MB
pub fn file_size_mb(path: &Path) -> u64 {
    path.metadata().map(|m| m.len() / (1024 * 1024)).unwrap_or(0)
}
