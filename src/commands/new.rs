use anyhow::{bail, Result};
use iocraft::prelude::*;

use crate::core::{config, mcu_db, template};
use crate::ui::{self, Entry, Header, Section, SectionVariant, StatusLine, StatusVariant};

pub fn run(name: &str, mcu_id: &str, project_type: &str, toolchain: Option<&str>) -> Result<()> {
    // 1. Lookup MCU
    let mcu = match mcu_db::lookup(mcu_id) {
        Some(m) => m,
        None => {
            let supported = mcu_db::supported_ids();
            bail!(
                "Unknown MCU '{}'. Supported: {}",
                mcu_id,
                supported.join(", ")
            );
        }
    };

    // 2. Parse project type
    let ptype = template::ProjectType::from_str(project_type)?;

    // 3. Check output directory
    let output_dir = std::env::current_dir()?.join(name);
    if output_dir.exists() {
        bail!("Directory '{}' already exists.", name);
    }

    // 4. Determine toolchain
    let (tc_vendor, tc_version) = if let Some(spec) = toolchain {
        crate::commands::toolchain::parse_spec(spec)?
    } else {
        // Use default from config or MCU vendor
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

    // 5. Show project info
    ui::render(element! {
        View(flex_direction: FlexDirection::Column) {
            Header(
                title: "embtool new".to_string(),
            )
            Section(title: format!("{}", name)) {
                Entry(label: "MCU".to_string(), value: format!(
                    "{} ({}, {}MHz, {} Flash, {} RAM)",
                    mcu.part_number, mcu.core, mcu.clock_mhz, mcu.flash_str(), mcu.ram_str()
                ))
                Entry(label: "Toolchain".to_string(), value: format!("{}:{}", tc_vendor, tc_version))
                Entry(label: "Type".to_string(), value: ptype.label().to_string())
            }
        }
    });

    // 6. Generate
    let opts = template::GenerateOptions {
        name,
        mcu,
        project_type: ptype,
        toolchain_vendor: &tc_vendor,
        toolchain_version: &tc_version,
        output_dir: &output_dir,
    };

    let files = template::generate_project(&opts)?;

    // 7. Show file tree
    println!();
    ui::render(element! {
        Section(title: "Generated".to_string(), variant: SectionVariant::Success) {
            #(files.iter().enumerate().map(|(i, f)| {
                let is_last = i == files.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                element! {
                    Text(
                        content: format!("   {}{}", prefix, f.display()),
                        color: Some(Color::DarkGrey),
                    )
                }
            }))
        }
    });

    println!();
    ui::render(element! {
        StatusLine(
            icon: "✓".to_string(),
            message: format!("Project created! Next steps:"),
            variant: StatusVariant::Success,
        )
    });
    ui::render(element! {
        View(flex_direction: FlexDirection::Column, margin_left: 3) {
            Text(content: format!("cd {}", name), color: Some(Color::Cyan))
            Text(content: "embtool setup".to_string(), color: Some(Color::Cyan))
            Text(content: "embtool build".to_string(), color: Some(Color::Cyan))
        }
    });

    Ok(())
}
