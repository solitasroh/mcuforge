use anyhow::{bail, Result};
use dialoguer::{Input, Select, Confirm};
use iocraft::prelude::*;

use crate::core::{cmake_provider, config, mcu_db, template::{self, GITIGNORE, CLANG_FORMAT_CONFIG, CLANG_TIDY_CONFIG}};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant};

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;

    if cwd.join("embtool.toml").exists() {
        bail!("embtool.toml already exists. Use 'embtool setup' to update.");
    }

    ui::render(element! {
        Header(
            title: "embtool init".to_string(),
            subtitle: Some(cwd.display().to_string()),
        )
    });
    println!();

    // 1. Project name
    let dir_name = cwd.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    let name: String = Input::new()
        .with_prompt("Project name")
        .default(dir_name)
        .interact_text()?;

    // 2. MCU selection
    let mcus = mcu_db::list_all();
    let mcu_labels: Vec<String> = mcus.iter().map(|m| {
        format!("{:<6} {} ({}MHz, {} Flash, {} RAM)",
            m.id, m.part_number, m.clock_mhz, m.flash_str(), m.ram_str())
    }).collect();

    let mcu_idx = Select::new()
        .with_prompt("Target MCU")
        .items(&mcu_labels)
        .default(0)
        .interact()?;
    let mcu = mcus[mcu_idx];

    // 3. Project type
    let types = ["application", "bootloader", "library"];
    let type_idx = Select::new()
        .with_prompt("Project type")
        .items(&types)
        .default(0)
        .interact()?;
    let ptype = template::ProjectType::from_str(types[type_idx])?;

    // 4. Toolchain
    let cfg = config::load()?;
    let default_tc = cfg.toolchain.default.clone()
        .unwrap_or_else(|| format!("{}:latest", mcu.vendor));
    let tc_spec: String = Input::new()
        .with_prompt("Toolchain (vendor:version)")
        .default(default_tc)
        .interact_text()?;

    let (tc_vendor, tc_version) = if let Some((v, ver)) = tc_spec.split_once(':') {
        (v.to_string(), ver.to_string())
    } else if let Some((v, ver)) = tc_spec.split_once('-') {
        (v.to_string(), ver.to_string())
    } else {
        (mcu.vendor.to_string(), tc_spec)
    };

    // 5. CMake version
    let cmake_ver: String = Input::new()
        .with_prompt("CMake version")
        .default(cmake_provider::DEFAULT_CMAKE_VERSION.to_string())
        .interact_text()?;

    // 6. clang tools
    let use_clang = Confirm::new()
        .with_prompt("Add clang-format & clang-tidy?")
        .default(true)
        .interact()?;

    let clang_ver = if use_clang {
        let ver: String = Input::new()
            .with_prompt("clang tools version")
            .default("18".to_string())
            .interact_text()?;
        Some(ver)
    } else {
        None
    };

    println!();

    // Show summary
    ui::render(element! {
        Section(title: name.clone()) {
            Entry(label: "MCU".to_string(), value: format!(
                "{} ({}, {}MHz, {} Flash, {} RAM)",
                mcu.part_number, mcu.core, mcu.clock_mhz, mcu.flash_str(), mcu.ram_str()
            ))
            Entry(label: "Type".to_string(), value: ptype.label().to_string())
            Entry(label: "Toolchain".to_string(), value: format!("{}:{}", tc_vendor, tc_version))
            Entry(label: "CMake".to_string(), value: cmake_ver.clone())
            Entry(label: "clang-tools".to_string(), value:
                clang_ver.as_deref().unwrap_or("(none)").to_string()
            )
        }
    });
    println!();

    let confirm = Confirm::new()
        .with_prompt("Create embtool.toml?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("Aborted.");
        return Ok(());
    }

    // Generate files
    let mut created = Vec::new();

    // 1. embtool.toml
    let mut toml_content = format!(
        r#"[project]
name = "{name}"
type = "{ptype}"

[target]
mcu = "{define}"
core = "{core}"
fpu = "{fpu}"
flash = "{flash}"
ram = "{ram}"

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
        fpu = mcu.fpu,
        flash = mcu.flash_str(),
        ram = mcu.ram_str(),
        vendor = tc_vendor,
        version = tc_version,
        cmake = cmake_ver,
    );

    if let Some(ref cv) = clang_ver {
        toml_content.push_str(&format!(
            r#"
[tools.clang-format]
version = "{v}"

[tools.clang-tidy]
version = "{v}"
"#, v = cv));
    }

    std::fs::write(cwd.join("embtool.toml"), &toml_content)?;
    created.push("embtool.toml");

    // 2. arm-toolchain.cmake
    if !cwd.join("arm-toolchain.cmake").exists() {
        let cmake_content = template::generate_toolchain_cmake(&tc_vendor, &tc_version);
        std::fs::write(cwd.join("arm-toolchain.cmake"), cmake_content)?;
        created.push("arm-toolchain.cmake");
    }

    // 3. .gitignore
    if !cwd.join(".gitignore").exists() {
        std::fs::write(cwd.join(".gitignore"), GITIGNORE)?;
        created.push(".gitignore");
    }

    // 4. .clangd
    if !cwd.join(".clangd").exists() {
        let clangd = template::generate_clangd_config(mcu.define, mcu.core, mcu.fpu);
        std::fs::write(cwd.join(".clangd"), clangd)?;
        created.push(".clangd");
    }

    // 5. .clang-format / .clang-tidy
    if clang_ver.is_some() {
        if !cwd.join(".clang-format").exists() {
            std::fs::write(cwd.join(".clang-format"), CLANG_FORMAT_CONFIG)?;
            created.push(".clang-format");
        }
        if !cwd.join(".clang-tidy").exists() {
            std::fs::write(cwd.join(".clang-tidy"), CLANG_TIDY_CONFIG)?;
            created.push(".clang-tidy");
        }
    }

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


