use anyhow::{bail, Result};
use iocraft::prelude::*;

use crate::core::{cmake_provider, config, mcu_db, template};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant};

pub fn run(
    mcu_id: &str,
    project_type: &str,
    toolchain: Option<&str>,
    cmake_version: Option<&str>,
    with_clang: Option<&str>,
) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if already initialized
    if cwd.join("embtool.toml").exists() {
        bail!("embtool.toml already exists. Use 'embtool setup' to update.");
    }

    // Lookup MCU
    let mcu = match mcu_db::lookup(mcu_id) {
        Some(m) => m,
        None => {
            let supported = mcu_db::supported_ids();
            bail!("Unknown MCU '{}'. Supported: {}", mcu_id, supported.join(", "));
        }
    };

    let ptype = template::ProjectType::from_str(project_type)?;

    // Project name from directory name
    let name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    // Resolve toolchain
    let (tc_vendor, tc_version) = if let Some(spec) = toolchain {
        crate::commands::toolchain::parse_spec(spec)?
    } else {
        let cfg = config::load()?;
        if let Some(default) = &cfg.toolchain.default {
            if let Some((v, ver)) = default.split_once('-') {
                (v.to_string(), ver.to_string())
            } else {
                (mcu.vendor.to_string(), "latest".to_string())
            }
        } else {
            (mcu.vendor.to_string(), "latest".to_string())
        }
    };

    let cmake_ver = cmake_version.unwrap_or(cmake_provider::DEFAULT_CMAKE_VERSION);

    // Show info
    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(title: "embtool init".to_string())
            Section(title: name.clone()) {
                Entry(label: "MCU".to_string(), value: format!(
                    "{} ({}, {}MHz, {} Flash, {} RAM)",
                    mcu.part_number, mcu.core, mcu.clock_mhz, mcu.flash_str(), mcu.ram_str()
                ))
                Entry(label: "Toolchain".to_string(), value: format!("{}:{}", tc_vendor, tc_version))
                Entry(label: "CMake".to_string(), value: cmake_ver.to_string())
                Entry(label: "Type".to_string(), value: ptype.label().to_string())
            }
        }
    });

    // Generate only config files (not src/main.c etc. — they already exist)
    let mut created = Vec::new();

    // 1. embtool.toml
    let mut toml_content = format!(
        r#"[project]
name = "{name}"
type = "{ptype}"

[target]
mcu = "{define}"
core = "{core}"

[toolchain]
vendor = "{vendor}"
version = "{version}"

[cmake]
version = "{cmake}"
"#,
        name = name,
        ptype = ptype.label(),
        define = mcu.define,
        core = mcu.core,
        vendor = tc_vendor,
        version = tc_version,
        cmake = cmake_ver,
    );

    if let Some(clang_ver) = with_clang {
        toml_content.push_str(&format!(
            r#"
[tools.clang-format]
version = "{v}"

[tools.clang-tidy]
version = "{v}"
"#,
            v = clang_ver,
        ));
    }

    std::fs::write(cwd.join("embtool.toml"), &toml_content)?;
    created.push("embtool.toml");

    // 2. arm-toolchain.cmake (always)
    let opts = template::GenerateOptions {
        name: &name,
        mcu,
        project_type: ptype,
        toolchain_vendor: &tc_vendor,
        toolchain_version: &tc_version,
        output_dir: &cwd,
    };
    if !cwd.join("arm-toolchain.cmake").exists() {
        // Reuse template generator for cmake file
        let cmake_content = gen_toolchain_cmake(&tc_vendor, &tc_version);
        std::fs::write(cwd.join("arm-toolchain.cmake"), cmake_content)?;
        created.push("arm-toolchain.cmake");
    }

    // 3. .gitignore (if not exists)
    if !cwd.join(".gitignore").exists() {
        std::fs::write(cwd.join(".gitignore"), GITIGNORE)?;
        created.push(".gitignore");
    }

    // 4. .clang-format / .clang-tidy (if --with-clang)
    if with_clang.is_some() {
        if !cwd.join(".clang-format").exists() {
            std::fs::write(cwd.join(".clang-format"), CLANG_FORMAT_CONFIG)?;
            created.push(".clang-format");
        }
        if !cwd.join(".clang-tidy").exists() {
            std::fs::write(cwd.join(".clang-tidy"), CLANG_TIDY_CONFIG)?;
            created.push(".clang-tidy");
        }
    }

    // Show results
    println!();
    ui::render(element! {
        Section(title: "Created".to_string(), variant: SectionVariant::Success) {
            #(created.iter().enumerate().map(|(i, f)| {
                let is_last = i == created.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                element! {
                    Text(content: format!("   {}{}", prefix, f), color: Some(Color::DarkGrey))
                }
            }))
        }
    });

    println!();
    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: "Initialized! Next:".to_string(),
            variant: StatusVariant::Success,
        )
    });
    ui::render(element! {
        View(flex_direction: FlexDirection::Column, margin_left: 3) {
            Text(content: "embtool setup".to_string(), color: Some(Color::Cyan))
            Text(content: "embtool build".to_string(), color: Some(Color::Cyan))
        }
    });

    Ok(())
}

fn gen_toolchain_cmake(vendor: &str, version: &str) -> String {
    let tc_id = format!("{}-{}", vendor, version);
    format!(
        r#"# Auto-generated by embtool — DO NOT EDIT
# Run 'embtool setup' to regenerate

cmake_minimum_required(VERSION 3.20)

if(WIN32)
    set(EMBTOOL_HOME "$ENV{{USERPROFILE}}/.embtool")
else()
    set(EMBTOOL_HOME "$ENV{{HOME}}/.embtool")
endif()

set(EMBTOOL_TOOLCHAIN_ID "{tc_id}")
set(ARM_TOOLCHAIN_ROOT "${{EMBTOOL_HOME}}/toolchains/${{EMBTOOL_TOOLCHAIN_ID}}")
set(ARM_TOOLCHAIN_PATH "${{ARM_TOOLCHAIN_ROOT}}/bin/")

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR arm)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_C_COMPILER "${{ARM_TOOLCHAIN_PATH}}arm-none-eabi-gcc")
set(CMAKE_CXX_COMPILER "${{ARM_TOOLCHAIN_PATH}}arm-none-eabi-g++")
set(CMAKE_ASM_COMPILER "${{ARM_TOOLCHAIN_PATH}}arm-none-eabi-gcc")
set(CMAKE_AR "${{ARM_TOOLCHAIN_PATH}}arm-none-eabi-ar")
set(CMAKE_OBJCOPY "${{ARM_TOOLCHAIN_PATH}}arm-none-eabi-objcopy" CACHE INTERNAL "")
set(CMAKE_SIZE "${{ARM_TOOLCHAIN_PATH}}arm-none-eabi-size" CACHE INTERNAL "")

set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
"#,
        tc_id = tc_id,
    )
}

const GITIGNORE: &str = r#"# Build output
build/
cmake-build-*/

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# embtool cache
.embtool-cache/

# Compiled output
*.o
*.d
*.elf
*.bin
*.hex
*.map
*.a

# OS
.DS_Store
Thumbs.db
"#;

const CLANG_FORMAT_CONFIG: &str = r#"---
BasedOnStyle: LLVM
IndentWidth: 4
TabWidth: 4
UseTab: Never
ColumnLimit: 120
BreakBeforeBraces: Allman
AllowShortFunctionsOnASingleLine: Empty
AllowShortIfStatementsOnASingleLine: false
AllowShortLoopsOnASingleLine: false
SortIncludes: true
IncludeBlocks: Regroup
"#;

const CLANG_TIDY_CONFIG: &str = r#"---
Checks: >
  -*,
  bugprone-*,
  cert-*,
  clang-analyzer-*,
  misc-*,
  modernize-*,
  performance-*,
  readability-*,
  -modernize-use-trailing-return-type,
  -readability-magic-numbers,
  -cert-err58-cpp

WarningsAsErrors: ''
HeaderFilterRegex: 'src/.*'

CheckOptions:
  - key: readability-identifier-naming.FunctionCase
    value: camelBack
  - key: readability-identifier-naming.VariableCase
    value: camelBack
  - key: readability-identifier-naming.GlobalConstantCase
    value: UPPER_CASE
"#;
