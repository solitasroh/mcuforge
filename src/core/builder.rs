use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{bail, Context, Result};

use crate::core::project::ProjectConfig;
use crate::utils::paths;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(Self::Debug),
            "release" => Ok(Self::Release),
            _ => bail!("Invalid build profile '{}'. Use 'debug' or 'release'.", s),
        }
    }

    pub fn cmake_type(&self) -> &str {
        match self {
            Self::Debug => "Debug",
            Self::Release => "Release",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Debug => "Debug",
            Self::Release => "Release",
        }
    }
}

#[derive(Debug)]
pub struct BuildResult {
    pub elf_path: PathBuf,
    pub bin_path: Option<PathBuf>,
    pub hex_path: Option<PathBuf>,
    pub flash_used: u32,
    pub flash_total: u32,
    pub ram_used: u32,
    pub ram_total: u32,
    pub build_time_secs: f64,
}

impl BuildResult {
    pub fn flash_pct(&self) -> f64 {
        if self.flash_total == 0 { return 0.0; }
        (self.flash_used as f64 / self.flash_total as f64) * 100.0
    }

    pub fn ram_pct(&self) -> f64 {
        if self.ram_total == 0 { return 0.0; }
        (self.ram_used as f64 / self.ram_total as f64) * 100.0
    }
}

/// Run cmake configure + build
pub fn build(
    project_dir: &Path,
    proj: &ProjectConfig,
    profile: BuildProfile,
    verbose: bool,
    clean: bool,
) -> Result<BuildResult> {
    let start = Instant::now();
    let build_dir = project_dir.join("build");

    // Clean if requested
    if clean && build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)
            .context("Failed to clean build directory")?;
    }

    std::fs::create_dir_all(&build_dir)?;

    // Resolve cmake path
    let cmake_bin = find_cmake(&proj.cmake.version)?;

    // CMake configure
    let toolchain_file = project_dir.join("arm-toolchain.cmake");
    if !toolchain_file.exists() {
        bail!("arm-toolchain.cmake not found. Run 'embtool setup' first.");
    }

    let mut configure = Command::new(&cmake_bin);
    configure
        .current_dir(project_dir)
        .arg("-B").arg("build")
        .arg(format!("-DCMAKE_BUILD_TYPE={}", profile.cmake_type()))
        .arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain_file.display()))
        .arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON");

    if verbose {
        configure.arg("-DCMAKE_VERBOSE_MAKEFILE=ON");
    }

    let configure_out = configure.output()
        .context("Failed to run cmake configure. Is cmake installed?")?;

    if !configure_out.status.success() {
        let stderr = String::from_utf8_lossy(&configure_out.stderr);
        let stdout = String::from_utf8_lossy(&configure_out.stdout);
        bail!("CMake configure failed:\n{}\n{}", stdout, stderr);
    }

    // CMake build
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut build_cmd = Command::new(&cmake_bin);
    build_cmd
        .current_dir(project_dir)
        .arg("--build").arg("build")
        .arg("--").arg(format!("-j{}", num_cpus));

    if verbose {
        build_cmd.arg("VERBOSE=1");
    }

    let build_out = build_cmd.output()
        .context("Failed to run cmake build")?;

    if !build_out.status.success() {
        let stderr = String::from_utf8_lossy(&build_out.stderr);
        let stdout = String::from_utf8_lossy(&build_out.stdout);
        bail!("Build failed:\n{}\n{}", stdout, stderr);
    }

    let build_time = start.elapsed().as_secs_f64();

    // Find the ELF output
    let elf_path = build_dir.join(format!("{}.elf", proj.project.name));
    if !elf_path.exists() {
        bail!("Expected ELF not found at {}. Check CMakeLists.txt output name.", elf_path.display());
    }

    // Parse size
    let toolchain_bin = paths::toolchain_path(&proj.toolchain.vendor, &proj.toolchain.version)?
        .join("bin");
    let (flash_used, ram_used) = parse_size(&toolchain_bin, &elf_path)?;

    // Get MCU flash/ram total from project config
    let flash_total = parse_size_spec(&proj.target.flash);
    let ram_total = parse_size_spec(&proj.target.ram);

    // Generate .bin and .hex
    let bin_path = objcopy(&toolchain_bin, &elf_path, "binary")?;
    let hex_path = objcopy(&toolchain_bin, &elf_path, "ihex")?;

    // Symlink compile_commands.json to project root for clangd
    let cc_src = build_dir.join("compile_commands.json");
    let cc_dst = project_dir.join("compile_commands.json");
    if cc_src.exists() && !cc_dst.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&cc_src, &cc_dst).ok();
        #[cfg(windows)]
        std::fs::copy(&cc_src, &cc_dst).ok();
    }

    Ok(BuildResult {
        elf_path,
        bin_path: Some(bin_path),
        hex_path: Some(hex_path),
        flash_used,
        flash_total,
        ram_used,
        ram_total,
        build_time_secs: build_time,
    })
}

/// Find cmake binary (embtool-managed or system)
fn find_cmake(version_spec: &str) -> Result<PathBuf> {
    use crate::core::cmake_provider::CmakeProvider;

    // Try embtool-managed cmake first
    let resolved = CmakeProvider::resolve_version(version_spec)?;
    let embtool_home = paths::embtool_home()?;
    let cmake_dir = embtool_home.join("cmake").join(&resolved);
    let cmake_bin = cmake_dir.join("bin").join(if cfg!(windows) { "cmake.exe" } else { "cmake" });

    if cmake_bin.exists() {
        return Ok(cmake_bin);
    }

    // Fallback to system cmake
    if let Ok(output) = Command::new("cmake").arg("--version").output() {
        if output.status.success() {
            return Ok(PathBuf::from("cmake"));
        }
    }

    bail!("CMake {} not found. Run 'embtool setup' or 'embtool cmake install {}'.", version_spec, version_spec)
}

/// Run arm-none-eabi-size and parse output
fn parse_size(toolchain_bin: &Path, elf: &Path) -> Result<(u32, u32)> {
    let size_bin = toolchain_bin.join(if cfg!(windows) { "arm-none-eabi-size.exe" } else { "arm-none-eabi-size" });

    let output = Command::new(&size_bin)
        .arg(elf)
        .output()
        .with_context(|| format!("Failed to run {}", size_bin.display()))?;

    if !output.status.success() {
        bail!("arm-none-eabi-size failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_size_output(&stdout)
}

/// Parse arm-none-eabi-size output:
/// ```
///    text    data     bss     dec     hex filename
///   42768    2544    9744   55056    d710 build/app.elf
/// ```
/// Flash = text + data, RAM = data + bss
pub fn parse_size_output(output: &str) -> Result<(u32, u32)> {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 {
        bail!("Unexpected size output: {}", output);
    }

    let data_line = lines[1].trim();
    let parts: Vec<&str> = data_line.split_whitespace().collect();
    if parts.len() < 4 {
        bail!("Cannot parse size line: {}", data_line);
    }

    let text: u32 = parts[0].parse().context("Failed to parse text section")?;
    let data: u32 = parts[1].parse().context("Failed to parse data section")?;
    let bss: u32 = parts[2].parse().context("Failed to parse bss section")?;

    let flash_used = text + data;
    let ram_used = data + bss;

    Ok((flash_used, ram_used))
}

/// Run objcopy to generate .bin or .hex
fn objcopy(toolchain_bin: &Path, elf: &Path, format: &str) -> Result<PathBuf> {
    let objcopy_bin = toolchain_bin.join(
        if cfg!(windows) { "arm-none-eabi-objcopy.exe" } else { "arm-none-eabi-objcopy" }
    );

    let ext = match format {
        "binary" => "bin",
        "ihex" => "hex",
        _ => bail!("Unknown objcopy format: {}", format),
    };

    let output_path = elf.with_extension(ext);

    let status = Command::new(&objcopy_bin)
        .arg("-O").arg(format)
        .arg(elf)
        .arg(&output_path)
        .status()
        .with_context(|| format!("Failed to run {}", objcopy_bin.display()))?;

    if !status.success() {
        bail!("objcopy failed for {} format", format);
    }

    Ok(output_path)
}

/// Parse size spec like "1M", "256K", "512K" → bytes
fn parse_size_spec(spec: &str) -> u32 {
    let s = spec.trim().to_uppercase();
    if s.is_empty() {
        return 0;
    }

    if s.ends_with("MB") || s.ends_with('M') {
        let num = s.trim_end_matches("MB").trim_end_matches('M');
        num.parse::<u32>().unwrap_or(0) * 1024 * 1024
    } else if s.ends_with("KB") || s.ends_with('K') {
        let num = s.trim_end_matches("KB").trim_end_matches('K');
        num.parse::<u32>().unwrap_or(0) * 1024
    } else {
        s.parse::<u32>().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_output() {
        let output = "   text    data     bss     dec     hex filename\n  42768    2544    9744   55056    d710 build/app.elf\n";
        let (flash, ram) = parse_size_output(output).unwrap();
        assert_eq!(flash, 42768 + 2544);  // text + data
        assert_eq!(ram, 2544 + 9744);     // data + bss
    }

    #[test]
    fn test_parse_size_output_small() {
        let output = "   text    data     bss     dec     hex filename\n    256      16      32     304     130 build/test.elf\n";
        let (flash, ram) = parse_size_output(output).unwrap();
        assert_eq!(flash, 272);
        assert_eq!(ram, 48);
    }

    #[test]
    fn test_parse_size_spec() {
        assert_eq!(parse_size_spec("1M"), 1048576);
        assert_eq!(parse_size_spec("256K"), 262144);
        assert_eq!(parse_size_spec("512KB"), 524288);
        assert_eq!(parse_size_spec("2MB"), 2097152);
        assert_eq!(parse_size_spec(""), 0);
        assert_eq!(parse_size_spec("1024"), 1024);
    }

    #[test]
    fn test_build_profile() {
        assert_eq!(BuildProfile::from_str("debug").unwrap(), BuildProfile::Debug);
        assert_eq!(BuildProfile::from_str("Release").unwrap(), BuildProfile::Release);
        assert!(BuildProfile::from_str("invalid").is_err());
    }

    #[test]
    fn test_build_result_pct() {
        let result = BuildResult {
            elf_path: PathBuf::new(),
            bin_path: None,
            hex_path: None,
            flash_used: 45312,
            flash_total: 1048576,
            ram_used: 12288,
            ram_total: 262144,
            build_time_secs: 1.0,
        };
        assert!((result.flash_pct() - 4.32).abs() < 0.1);
        assert!((result.ram_pct() - 4.69).abs() < 0.1);
    }
}
