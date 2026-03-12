use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::core::project::ProjectConfig;

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
    // Hooks
    copy_dir_if_exists(&cache_dir.join("hooks"), &hooks_dir)?;

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

/// Download skills package from GitHub release to cache
pub fn download_skills_package(version: Option<&str>) -> Result<PathBuf> {
    let cache_base = dirs::home_dir()
        .context("Cannot find home directory")?
        .join(".embtool")
        .join("claude-skills");

    let version = version.unwrap_or("latest");
    let cache_dir = cache_base.join(version);

    if cache_dir.exists() {
        return Ok(cache_dir);
    }

    // TODO: Implement GitHub API download
    // GET https://api.github.com/repos/solitasroh/mcuforge/releases/latest
    // Find asset matching claude-skills-v*.tar.gz
    // Download and extract to cache_dir

    anyhow::bail!(
        "Skills package v{} not found in cache (~/.embtool/claude-skills/{}).\n\
         Download from: https://github.com/solitasroh/mcuforge/releases\n\
         Extract to: {}",
        version,
        version,
        cache_dir.display()
    )
}

#[derive(Default)]
pub struct InstallReport {
    pub installed: Vec<String>,
    pub templates: Vec<String>,
    pub skipped: Vec<String>,
}

impl InstallReport {
    pub fn total(&self) -> usize {
        self.installed.len() + self.templates.len()
    }
}

/// Generate the [claude] section for embtool.toml
pub fn generate_claude_toml_section(
    gitlab_url: Option<&str>,
    gitlab_ssh_port: Option<u16>,
    op_url: Option<&str>,
    op_project_id: Option<u32>,
) -> String {
    let mut section = String::new();
    section.push_str("\n[claude]\nversion = \"1.0.0\"\n");
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
