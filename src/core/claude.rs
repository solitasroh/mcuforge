use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

use crate::core::{config, project::ProjectConfig};
use crate::utils::paths;

/// Template rendering context for skill templates
pub struct TemplateContext {
    vars: HashMap<String, String>,
}

impl TemplateContext {
    pub fn from_config(config: &ProjectConfig) -> Self {
        let mut vars = HashMap::new();
        vars.insert("project_name".into(), config.project.name.clone());
        vars.insert("mcu".into(), config.target.mcu.clone());
        vars.insert("core".into(), config.target.core.clone());
        vars.insert("fpu".into(), config.target.fpu.clone());
        vars.insert("flash_kb".into(), parse_size_kb(&config.target.flash));
        vars.insert("ram_kb".into(), parse_size_kb(&config.target.ram));

        if let Some(ref claude) = config.claude {
            if let Some(ref gitlab) = claude.gitlab {
                vars.insert("gitlab_url".into(), gitlab.url.clone());
                // Extract host from URL (e.g., "http://10.10.20.32" → "10.10.20.32")
                let host = gitlab
                    .url
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .trim_end_matches('/')
                    .to_string();
                vars.insert("gitlab_url_host".into(), host);
                vars.insert(
                    "gitlab_ssh_port".into(),
                    gitlab.ssh_port.unwrap_or(22).to_string(),
                );
                vars.insert(
                    "target_branch".into(),
                    gitlab
                        .target_branch
                        .clone()
                        .unwrap_or_else(|| "develop".into()),
                );
                if let Some(ref path) = gitlab.project_path {
                    vars.insert("gitlab_project_path".into(), path.clone());
                }
            }
            if let Some(ref op) = claude.openproject {
                vars.insert("op_url".into(), op.url.clone());
                vars.insert("op_project_id".into(), op.project_id.to_string());
                if let Some(ref codes) = op.product_codes {
                    vars.insert("product_codes".into(), codes.join(", "));
                }
            }
            if let Some(ref mem) = claude.memory {
                if let Some(ref ss) = mem.stack_size {
                    vars.insert("stack_size".into(), ss.clone());
                    if let Some(kb) = hex_to_kb(ss) {
                        vars.insert("stack_size_kb".into(), kb);
                    }
                }
                if let Some(ref start) = mem.app_flash_start {
                    vars.insert("app_flash_start".into(), start.clone());
                    if let Some(kb) = hex_to_kb(start) {
                        let flash_kb: u32 =
                            parse_size_kb(&config.target.flash).parse().unwrap_or(0);
                        let offset_kb: u32 = kb.parse::<f32>().unwrap_or(0.0) as u32;
                        vars.insert("app_flash_kb".into(), (flash_kb - offset_kb).to_string());
                    }
                }
            }
        }
        Self { vars }
    }

    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

fn parse_size_kb(s: &str) -> String {
    let s = s.trim().to_uppercase();
    if let Some(num) = s.strip_suffix("KB") {
        num.trim().to_string()
    } else if let Some(num) = s.strip_suffix("K") {
        num.trim().to_string()
    } else if let Some(num) = s.strip_suffix("MB") {
        if let Ok(n) = num.trim().parse::<u32>() {
            (n * 1024).to_string()
        } else {
            s
        }
    } else if let Some(num) = s.strip_suffix("M") {
        if let Ok(n) = num.trim().parse::<u32>() {
            (n * 1024).to_string()
        } else {
            s
        }
    } else {
        s
    }
}

fn hex_to_kb(s: &str) -> Option<String> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        if let Ok(bytes) = u32::from_str_radix(hex, 16) {
            let kb = bytes as f32 / 1024.0;
            return Some(format!("{}", kb as u32));
        }
    }
    None
}

/// Install skills from cached package to project directory
pub fn install_skills(
    project_dir: &Path,
    config: &ProjectConfig,
    cache_dir: &Path,
    force: bool,
) -> Result<InstallReport> {
    let ctx = TemplateContext::from_config(config);
    let claude_dir = project_dir.join(".claude");
    let skills_dir = claude_dir.join("skills");
    let agents_dir = claude_dir.join("agents");
    let hooks_dir = claude_dir.join("hooks");

    fs::create_dir_all(&skills_dir)?;
    fs::create_dir_all(&agents_dir)?;
    fs::create_dir_all(&hooks_dir)?;

    let mut report = InstallReport::default();

    let claude_cfg = config.claude.clone().unwrap_or_default();
    let skills_cfg = &claude_cfg.skills;

    // Universal skills
    if skills_cfg.universal.unwrap_or(true) {
        install_category(
            &cache_dir.join("universal"),
            &skills_dir,
            force,
            &mut report,
        )?;
    }

    // Embedded-C skills
    if skills_cfg.embedded_c.unwrap_or(true) {
        install_category(
            &cache_dir.join("embedded-c"),
            &skills_dir,
            force,
            &mut report,
        )?;
    }

    // Template skills
    let template_dir = cache_dir.join("templates");
    if template_dir.exists() {
        for entry in fs::read_dir(&template_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let skill_name = entry.file_name().to_string_lossy().to_string();

            match skills_cfg.overrides.get(&skill_name) {
                Some(toml::Value::Boolean(false)) => continue,
                Some(toml::Value::String(s)) if s == "local" => {
                    report.skipped.push(skill_name);
                    continue;
                }
                _ => {}
            }

            install_template_skill(&entry.path(), &skills_dir.join(&skill_name), &ctx, force)?;
            report.templates.push(skill_name);
        }
    }

    // Agents
    copy_dir_if_exists(&cache_dir.join("agents"), &agents_dir)?;
    report.agents_installed = count_files_in_dir(&agents_dir, "md");

    // Hooks
    copy_dir_if_exists(&cache_dir.join("hooks"), &hooks_dir)?;
    report.hooks_installed = count_files_in_dir(&hooks_dir, "sh");

    // Install settings.json (hooks configuration)
    report.settings_updated = install_settings_json(cache_dir, &claude_dir)?;

    // Update CLAUDE.md skills section
    update_claude_md_skills_section(project_dir)?;

    // Write manifest
    write_installed_manifest(&claude_dir, &report)?;

    Ok(report)
}

fn install_category(
    src: &Path,
    dest: &Path,
    force: bool,
    report: &mut InstallReport,
) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let skill_name = entry.file_name().to_string_lossy().to_string();
            let dest_skill = dest.join(&skill_name);
            if dest_skill.exists() && !force {
                report.skipped.push(skill_name);
                continue;
            }
            copy_dir_recursive(&entry.path(), &dest_skill)?;
            report.installed.push(skill_name);
        }
    }
    Ok(())
}

fn install_template_skill(
    src: &Path,
    dest: &Path,
    ctx: &TemplateContext,
    _force: bool,
) -> Result<()> {
    fs::create_dir_all(dest)?;
    for file_path in walkdir_files(src)? {
        let rel = file_path
            .strip_prefix(src)
            .context("Failed to strip prefix")?;
        let dest_path = if rel.to_string_lossy().ends_with(".tmpl") {
            let name = rel.to_string_lossy();
            dest.join(name.trim_end_matches(".tmpl"))
        } else {
            dest.join(rel)
        };

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if file_path.to_string_lossy().ends_with(".tmpl") {
            let content = fs::read_to_string(&file_path)?;
            let rendered = ctx.render(&content);
            fs::write(&dest_path, rendered)?;
        } else {
            fs::copy(&file_path, &dest_path)?;
        }
    }
    Ok(())
}

fn copy_dir_if_exists(src: &Path, dest: &Path) -> Result<()> {
    if src.exists() {
        copy_dir_recursive(src, dest)?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

fn walkdir_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                walk(&entry.path(), files)?;
            } else {
                files.push(entry.path());
            }
        }
        Ok(())
    }
    walk(dir, &mut files)?;
    Ok(files)
}

const SKILLS_BEGIN_MARKER: &str = "<!-- BEGIN MCUFORGE SKILLS -->";
const SKILLS_END_MARKER: &str = "<!-- END MCUFORGE SKILLS -->";

fn update_claude_md_skills_section(project_dir: &Path) -> Result<()> {
    let claude_md = project_dir.join("CLAUDE.md");
    let skills_section =
        "## Skills\n\nManaged by mcuforge. Run `mcuforge claude list` for details.\n";

    if claude_md.exists() {
        let content = fs::read_to_string(&claude_md)?;
        if let (Some(begin), Some(end)) = (
            content.find(SKILLS_BEGIN_MARKER),
            content.find(SKILLS_END_MARKER),
        ) {
            let end = end + SKILLS_END_MARKER.len();
            let mut new_content = String::new();
            new_content.push_str(&content[..begin]);
            new_content.push_str(SKILLS_BEGIN_MARKER);
            new_content.push('\n');
            new_content.push_str(skills_section);
            new_content.push_str(SKILLS_END_MARKER);
            new_content.push_str(&content[end..]);
            fs::write(&claude_md, new_content)?;
        } else {
            let mut content = content;
            content.push_str("\n\n");
            content.push_str(SKILLS_BEGIN_MARKER);
            content.push('\n');
            content.push_str(skills_section);
            content.push_str(SKILLS_END_MARKER);
            content.push('\n');
            fs::write(&claude_md, content)?;
        }
    }
    Ok(())
}

fn count_files_in_dir(dir: &Path, extension: &str) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == extension)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Install settings.json by merging hooks section into existing settings
fn install_settings_json(cache_dir: &Path, claude_dir: &Path) -> Result<bool> {
    let tmpl_path = cache_dir.join("settings.json.tmpl");
    if !tmpl_path.exists() {
        return Ok(false);
    }

    let settings_path = claude_dir.join("settings.json");
    let new_content = fs::read_to_string(&tmpl_path)?;
    let new_json: serde_json::Value =
        serde_json::from_str(&new_content).context("Failed to parse settings.json.tmpl")?;

    if settings_path.exists() {
        // Merge: preserve existing settings, update hooks section only
        let existing_content = fs::read_to_string(&settings_path)?;
        let mut existing_json: serde_json::Value = serde_json::from_str(&existing_content)
            .context("Failed to parse existing settings.json")?;

        if let Some(hooks) = new_json.get("hooks") {
            existing_json
                .as_object_mut()
                .context("settings.json is not an object")?
                .insert("hooks".to_string(), hooks.clone());
        }

        fs::write(
            &settings_path,
            serde_json::to_string_pretty(&existing_json)?,
        )?;
    } else {
        // Create new
        fs::write(
            &settings_path,
            serde_json::to_string_pretty(&new_json)?,
        )?;
    }

    Ok(true)
}

fn write_installed_manifest(claude_dir: &Path, report: &InstallReport) -> Result<()> {
    let manifest = serde_json::json!({
        "installed_at": chrono_now_iso(),
        "skills": {
            "universal_and_embedded": &report.installed,
            "templates": &report.templates,
            "skipped": &report.skipped,
        }
    });
    let path = claude_dir.join("skills").join(".manifest.json");
    fs::write(path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

fn chrono_now_iso() -> String {
    // Simple ISO timestamp without chrono dependency
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}", now.as_secs())
}

const GITHUB_REPO: &str = "solitasroh/mcuforge";
const SKILLS_ASSET_PREFIX: &str = "claude-skills-v";

/// Generate cache key candidates: ["0.3.0", "v0.3.0"] or ["latest"]
fn cache_candidates(version_key: &str) -> Vec<String> {
    if version_key == "latest" {
        vec!["latest".into()]
    } else if let Some(stripped) = version_key.strip_prefix('v') {
        vec![version_key.into(), stripped.into()]
    } else {
        vec![version_key.into(), format!("v{}", version_key)]
    }
}

/// Download skills package from GitHub release to cache
pub fn download_skills_package(version: Option<&str>) -> Result<PathBuf> {
    let cache_base = paths::skills_cache_dir()?;
    let version_key = version.unwrap_or("latest");

    // Try cache hit with both "X.Y.Z" and "vX.Y.Z" variants
    for candidate_key in cache_candidates(version_key) {
        let cache_dir = cache_base.join(&candidate_key);
        if cache_dir.join("manifest.json").exists() {
            // Check if this is a pointer dir (Windows "latest" alias)
            let source_file = cache_dir.join(".source");
            if source_file.exists() {
                let real_dir = PathBuf::from(fs::read_to_string(&source_file)?.trim());
                if real_dir.join("manifest.json").exists() {
                    return Ok(real_dir);
                }
            }
            return Ok(cache_dir);
        }
    }

    let cache_dir = cache_base.join(version_key);

    // Resolve release info from GitHub API
    let (tag, asset_url, asset_size) = resolve_release(version)?;
    let resolved_dir = cache_base.join(&tag);

    // Check if resolved version is already cached
    if resolved_dir.join("manifest.json").exists() {
        // Symlink "latest" → resolved tag
        if version_key == "latest" && resolved_dir != cache_dir {
            symlink_or_copy_dir(&resolved_dir, &cache_dir)?;
        }
        return Ok(resolved_dir);
    }

    // Download tar.gz to temp file
    eprintln!("  Downloading skills package {}...", tag);
    let tar_gz_data = download_asset(&asset_url, asset_size)?;

    // Extract with flate2 + tar
    eprintln!("  Extracting to {}...", resolved_dir.display());
    fs::create_dir_all(&resolved_dir)?;
    extract_tar_gz(&tar_gz_data, &resolved_dir)?;

    // Verify manifest exists
    if !resolved_dir.join("manifest.json").exists() {
        fs::remove_dir_all(&resolved_dir).ok();
        bail!("Invalid skills package: manifest.json not found after extraction");
    }

    // Link "latest" → resolved tag
    if version_key == "latest" && resolved_dir != cache_dir {
        symlink_or_copy_dir(&resolved_dir, &cache_dir)?;
    }

    Ok(resolved_dir)
}

/// Resolve GitHub release → (tag, asset_download_url, size)
fn resolve_release(version: Option<&str>) -> Result<(String, String, u64)> {
    let api_url = match version {
        Some(v) if v != "latest" => format!(
            "https://api.github.com/repos/{}/releases/tags/v{}",
            GITHUB_REPO, v
        ),
        _ => format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        ),
    };

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&api_url)
        .header("User-Agent", "mcuforge")
        .header("Accept", "application/vnd.github+json")
        .send()
        .with_context(|| format!("Failed to connect to GitHub API: {}", api_url))?;

    if !resp.status().is_success() {
        bail!(
            "GitHub API error: HTTP {} for {}\nCheck network or release version.",
            resp.status(),
            api_url
        );
    }

    let release: serde_json::Value = resp.json().context("Failed to parse GitHub API response")?;

    let tag = release["tag_name"]
        .as_str()
        .context("Release missing tag_name")?
        .to_string();

    // Find skills asset
    let assets = release["assets"]
        .as_array()
        .context("Release missing assets")?;

    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");
        if name.starts_with(SKILLS_ASSET_PREFIX) && name.ends_with(".tar.gz") {
            let url = asset["browser_download_url"]
                .as_str()
                .context("Asset missing download URL")?
                .to_string();
            let size = asset["size"].as_u64().unwrap_or(0);
            return Ok((tag, url, size));
        }
    }

    bail!(
        "No skills asset ({}*.tar.gz) found in release {}.\n\
         Available assets: {}",
        SKILLS_ASSET_PREFIX,
        tag,
        assets
            .iter()
            .filter_map(|a| a["name"].as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Download asset with progress bar
fn download_asset(url: &str, total_size: u64) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::new();
    let mut resp = client
        .get(url)
        .header("User-Agent", "mcuforge")
        .send()
        .with_context(|| format!("Failed to download: {}", url))?;

    if !resp.status().is_success() {
        bail!("Download failed: HTTP {} from {}", resp.status(), url);
    }

    let actual_size = resp.content_length().unwrap_or(total_size);
    let pb = if !config::is_ci() && actual_size > 0 {
        let pb = ProgressBar::new(actual_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("   {bar:40.cyan/dim} {percent}% ({bytes}/{total_bytes})")
                .unwrap()
                .progress_chars("━╸─"),
        );
        Some(pb)
    } else {
        if actual_size > 0 {
            eprintln!(
                "   Downloading ({:.1} KB)...",
                actual_size as f64 / 1024.0
            );
        }
        None
    };

    let mut data = Vec::with_capacity(actual_size as usize);
    let mut downloaded: u64 = 0;
    loop {
        let mut buf = [0u8; 8192];
        let n = resp.read(&mut buf).context("Download interrupted")?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buf[..n]);
        downloaded += n as u64;
        if let Some(ref pb) = pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    Ok(data)
}

/// Extract tar.gz bytes into destination directory
fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Failed to read tar entry")?;
        let path = entry.path().context("Invalid entry path")?.into_owned();

        let dest_path = dest.join(&path);

        // Safety: prevent path traversal
        if !dest_path.starts_with(dest) {
            bail!("Path traversal detected in archive: {}", path.display());
        }

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            entry
                .unpack(&dest_path)
                .with_context(|| format!("Failed to extract: {}", path.display()))?;
        }
    }

    Ok(())
}

/// Create a symlink (Unix) or copy directory (Windows) for "latest" alias
fn symlink_or_copy_dir(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest).ok();
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dest)
            .with_context(|| format!("Failed to symlink {} → {}", dest.display(), src.display()))?;
    }

    #[cfg(windows)]
    {
        // Junction requires admin on older Windows; just write a pointer file
        fs::create_dir_all(dest)?;
        fs::write(
            dest.join(".source"),
            src.to_string_lossy().as_bytes(),
        )?;
        // Copy manifest.json so cache hit check works
        if src.join("manifest.json").exists() {
            fs::copy(src.join("manifest.json"), dest.join("manifest.json"))?;
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct InstallReport {
    pub installed: Vec<String>,
    pub templates: Vec<String>,
    pub skipped: Vec<String>,
    pub hooks_installed: usize,
    pub agents_installed: usize,
    pub settings_updated: bool,
}

impl InstallReport {
    pub fn total(&self) -> usize {
        self.installed.len() + self.templates.len()
    }
}

/// Install skills without project config (standalone mode)
/// Only installs universal + embedded-c skills, skips templates
pub fn install_skills_standalone(
    project_dir: &Path,
    cache_dir: &Path,
    force: bool,
) -> Result<InstallReport> {
    let claude_dir = project_dir.join(".claude");
    let skills_dir = claude_dir.join("skills");
    let agents_dir = claude_dir.join("agents");
    let hooks_dir = claude_dir.join("hooks");

    fs::create_dir_all(&skills_dir)?;
    fs::create_dir_all(&agents_dir)?;
    fs::create_dir_all(&hooks_dir)?;

    let mut report = InstallReport::default();

    // Universal + Embedded-C only (no template rendering needed)
    install_category(&cache_dir.join("universal"), &skills_dir, force, &mut report)?;
    install_category(&cache_dir.join("embedded-c"), &skills_dir, force, &mut report)?;

    // Agents
    copy_dir_if_exists(&cache_dir.join("agents"), &agents_dir)?;
    report.agents_installed = count_files_in_dir(&agents_dir, "md");

    // Hooks
    copy_dir_if_exists(&cache_dir.join("hooks"), &hooks_dir)?;
    report.hooks_installed = count_files_in_dir(&hooks_dir, "sh");

    // Install settings.json (hooks configuration)
    report.settings_updated = install_settings_json(cache_dir, &claude_dir)?;

    // Manifest
    write_installed_manifest(&claude_dir, &report)?;

    Ok(report)
}

/// Generate the [claude] section for embtool.toml
pub fn generate_claude_toml_section(
    gitlab_url: Option<&str>,
    gitlab_ssh_port: Option<u16>,
    op_url: Option<&str>,
    op_project_id: Option<u32>,
) -> String {
    let mut section = String::new();
    section.push_str("\n[claude]\nversion = \"2.0.0\"\n");
    section.push_str("\n[claude.skills]\nuniversal = true\nembedded_c = true\n");

    if let Some(url) = gitlab_url {
        section.push_str(&format!("\n[claude.gitlab]\nurl = \"{}\"\n", url));
        if let Some(port) = gitlab_ssh_port {
            section.push_str(&format!("ssh_port = {}\n", port));
        }
        section.push_str("target_branch = \"develop\"\n");
    }

    if let (Some(url), Some(id)) = (op_url, op_project_id) {
        section.push_str(&format!(
            "\n[claude.openproject]\nurl = \"{}\"\nproject_id = {}\n",
            url, id
        ));
    }

    section
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_kb() {
        assert_eq!(parse_size_kb("256KB"), "256");
        assert_eq!(parse_size_kb("256K"), "256");
        assert_eq!(parse_size_kb("1MB"), "1024");
        assert_eq!(parse_size_kb("1M"), "1024");
        assert_eq!(parse_size_kb("64KB"), "64");
    }

    #[test]
    fn test_hex_to_kb() {
        assert_eq!(hex_to_kb("0x0600"), Some("1".to_string()));
        assert_eq!(hex_to_kb("0x10000"), Some("64".to_string()));
        assert_eq!(hex_to_kb("invalid"), None);
    }

    #[test]
    fn test_template_render() {
        let mut vars = HashMap::new();
        vars.insert("mcu".into(), "MK10D7".into());
        vars.insert("flash_kb".into(), "256".into());
        let ctx = TemplateContext { vars };

        let result = ctx.render("Target: {{mcu}} with {{flash_kb}}KB Flash");
        assert_eq!(result, "Target: MK10D7 with 256KB Flash");
    }

    #[test]
    fn test_generate_claude_toml_section() {
        let section = generate_claude_toml_section(
            Some("http://10.10.20.32"),
            Some(8022),
            Some("http://10.10.20.32:8080"),
            Some(5),
        );
        assert!(section.contains("[claude.gitlab]"));
        assert!(section.contains("ssh_port = 8022"));
        assert!(section.contains("[claude.openproject]"));
        assert!(section.contains("project_id = 5"));
    }
}
