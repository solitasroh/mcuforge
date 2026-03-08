use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::core::mcu_db::McuInfo;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectType {
    Application,
    Bootloader,
    Library,
}

impl ProjectType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "application" | "app" => Ok(Self::Application),
            "bootloader" | "boot" => Ok(Self::Bootloader),
            "library" | "lib" => Ok(Self::Library),
            _ => anyhow::bail!("Unknown project type '{}'. Use: application, bootloader, library", s),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Bootloader => "bootloader",
            Self::Library => "library",
        }
    }
}

pub struct GenerateOptions<'a> {
    pub name: &'a str,
    pub mcu: &'a McuInfo,
    pub project_type: ProjectType,
    pub toolchain_vendor: &'a str,
    pub toolchain_version: &'a str,
    pub output_dir: &'a Path,
}

/// Generate a complete project
pub fn generate_project(opts: &GenerateOptions) -> Result<Vec<PathBuf>> {
    let dir = opts.output_dir;
    let mut created = Vec::new();

    std::fs::create_dir_all(dir)?;
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::create_dir_all(dir.join("system"))?;

    // 1. embtool.toml
    let embtool_toml = gen_embtool_toml(opts);
    write_file(dir, "embtool.toml", &embtool_toml, &mut created)?;

    // 2. CMakeLists.txt
    let cmake = gen_cmakelists(opts);
    write_file(dir, "CMakeLists.txt", &cmake, &mut created)?;

    // 3. arm-toolchain.cmake
    let toolchain_cmake = gen_toolchain_cmake(opts);
    write_file(dir, "arm-toolchain.cmake", &toolchain_cmake, &mut created)?;

    // 4. .gitignore
    write_file(dir, ".gitignore", GITIGNORE, &mut created)?;

    // 5. system/startup.c
    if opts.project_type != ProjectType::Library {
        let startup = gen_startup(opts);
        write_file(dir, "system/startup.c", &startup, &mut created)?;
    }

    // 6. system/linkerscript.ld
    if opts.project_type != ProjectType::Library {
        let linker = gen_linkerscript(opts);
        write_file(dir, "system/linkerscript.ld", &linker, &mut created)?;
    }

    // 7. system/system_{mcu}.c
    let system_c = gen_system_c(opts);
    let system_name = format!("system/system_{}.c", opts.mcu.define);
    write_file(dir, &system_name, &system_c, &mut created)?;

    // 8. src/main.c (not for library)
    if opts.project_type != ProjectType::Library {
        let main_c = gen_main_c(opts);
        write_file(dir, "src/main.c", &main_c, &mut created)?;
    }

    // 9. .clangd (LSP configuration for cross-compilation)
    let clangd = gen_clangd_config(opts);
    write_file(dir, ".clangd", &clangd, &mut created)?;

    Ok(created)
}

fn write_file(base: &Path, rel: &str, content: &str, created: &mut Vec<PathBuf>) -> Result<()> {
    let path = base.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    created.push(PathBuf::from(rel));
    Ok(())
}

fn gen_embtool_toml(opts: &GenerateOptions) -> String {
    format!(
        r#"[project]
name = "{name}"
type = "{ptype}"

[target]
mcu = "{define}"
core = "{core}"

[toolchain]
vendor = "{vendor}"
version = "{version}"
"#,
        name = opts.name,
        ptype = opts.project_type.label(),
        define = opts.mcu.define,
        core = opts.mcu.core,
        vendor = opts.toolchain_vendor,
        version = opts.toolchain_version,
    )
}

fn gen_cmakelists(opts: &GenerateOptions) -> String {
    let fpu_flags = match opts.mcu.fpu {
        "hard" => "-mfloat-abi=hard -mfpu=fpv4-sp-d16",
        "softfp" => "-mfloat-abi=softfp -mfpu=fpv4-sp-d16",
        _ => "-mfloat-abi=soft",
    };

    let output_type = match opts.project_type {
        ProjectType::Library => "add_library(${PROJECT_NAME} STATIC ${SOURCES})",
        _ => {
            "add_executable(${EXECUTABLE} ${SOURCES})"
        }
    };

    let linker_section = if opts.project_type != ProjectType::Library {
        format!(
            r#"
# Linker
set(LINKER_SCRIPT "${{CMAKE_SOURCE_DIR}}/system/linkerscript.ld")
target_link_options(${{EXECUTABLE}} PRIVATE
    -T${{LINKER_SCRIPT}}
    -mcpu={core} -mthumb {fpu}
    -Wl,--gc-sections
    -Wl,-Map=${{PROJECT_NAME}}.map
    -specs=nano.specs -specs=nosys.specs
)

# Post-build: generate .bin and .hex
add_custom_command(TARGET ${{EXECUTABLE}} POST_BUILD
    COMMAND ${{CMAKE_OBJCOPY}} -O binary ${{EXECUTABLE}} ${{PROJECT_NAME}}.bin
    COMMAND ${{CMAKE_OBJCOPY}} -O ihex ${{EXECUTABLE}} ${{PROJECT_NAME}}.hex
    COMMAND ${{CMAKE_SIZE}} ${{EXECUTABLE}}
    COMMENT "Generating .bin and .hex"
)"#,
            core = opts.mcu.core,
            fpu = fpu_flags,
        )
    } else {
        String::new()
    };

    format!(
        r#"# Auto-generated by embtool — {name}
cmake_minimum_required(VERSION 3.20)

set(CMAKE_TOOLCHAIN_FILE "${{CMAKE_SOURCE_DIR}}/arm-toolchain.cmake")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project({name} C ASM)
set(EXECUTABLE ${{PROJECT_NAME}}.elf)

# Target MCU flags
add_compile_options(
    -D{define}
    -mcpu={core}
    {fpu}
    -mthumb
    -ffunction-sections
    -fdata-sections
    -fno-common
    -fsigned-char
    -Wall -Wextra
    -Os
)

# Sources
file(GLOB_RECURSE SOURCES "src/*.c" "system/*.c")
{output_type}
{linker}
"#,
        name = opts.name,
        define = opts.mcu.define,
        core = opts.mcu.core,
        fpu = fpu_flags,
        output_type = output_type,
        linker = linker_section,
    )
}

fn gen_toolchain_cmake(opts: &GenerateOptions) -> String {
    generate_toolchain_cmake(opts.toolchain_vendor, opts.toolchain_version)
}

fn gen_startup(opts: &GenerateOptions) -> String {
    format!(
        r#"/**
 * Startup code for {part} ({define})
 * Auto-generated by embtool
 */

#include <stdint.h>

extern uint32_t _estack;
extern uint32_t _sidata, _sdata, _edata;
extern uint32_t _sbss, _ebss;

extern void SystemInit(void);
extern int main(void);

void Reset_Handler(void);
void Default_Handler(void);

/* Cortex-M4 core handlers */
void NMI_Handler(void)         __attribute__((weak, alias("Default_Handler")));
void HardFault_Handler(void)   __attribute__((weak, alias("Default_Handler")));
void MemManage_Handler(void)   __attribute__((weak, alias("Default_Handler")));
void BusFault_Handler(void)    __attribute__((weak, alias("Default_Handler")));
void UsageFault_Handler(void)  __attribute__((weak, alias("Default_Handler")));
void SVC_Handler(void)         __attribute__((weak, alias("Default_Handler")));
void DebugMon_Handler(void)    __attribute__((weak, alias("Default_Handler")));
void PendSV_Handler(void)      __attribute__((weak, alias("Default_Handler")));
void SysTick_Handler(void)     __attribute__((weak, alias("Default_Handler")));

/* Vector table */
__attribute__((section(".isr_vector")))
const void *vector_table[] = {{
    &_estack,
    Reset_Handler,
    NMI_Handler,
    HardFault_Handler,
    MemManage_Handler,
    BusFault_Handler,
    UsageFault_Handler,
    0, 0, 0, 0,
    SVC_Handler,
    DebugMon_Handler,
    0,
    PendSV_Handler,
    SysTick_Handler,
}};

void Reset_Handler(void) {{
    /* Copy .data from Flash to RAM */
    uint32_t *src = &_sidata;
    uint32_t *dst = &_sdata;
    while (dst < &_edata) {{
        *dst++ = *src++;
    }}

    /* Zero .bss */
    dst = &_sbss;
    while (dst < &_ebss) {{
        *dst++ = 0;
    }}

    SystemInit();
    main();

    while (1) {{}}
}}

void Default_Handler(void) {{
    while (1) {{}}
}}
"#,
        part = opts.mcu.part_number,
        define = opts.mcu.define,
    )
}

fn gen_linkerscript(opts: &GenerateOptions) -> String {
    let (flash_origin, flash_size) = match opts.project_type {
        ProjectType::Bootloader => ("0x00000000".to_string(), "32K".to_string()),
        _ => ("0x00000000".to_string(), format!("{}K", opts.mcu.flash_kb)),
    };

    format!(
        r#"/**
 * Linker script for {part} ({define})
 * Flash: {flash_kb}KB, RAM: {ram_kb}KB
 * Auto-generated by embtool
 */

MEMORY
{{
    FLASH (rx)  : ORIGIN = {flash_origin}, LENGTH = {flash_size}
    RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = {ram_kb}K
}}

_estack = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS
{{
    .isr_vector :
    {{
        . = ALIGN(4);
        KEEP(*(.isr_vector))
        . = ALIGN(4);
    }} > FLASH

    .text :
    {{
        . = ALIGN(4);
        *(.text)
        *(.text*)
        *(.rodata)
        *(.rodata*)
        . = ALIGN(4);
        _etext = .;
    }} > FLASH

    _sidata = LOADADDR(.data);

    .data :
    {{
        . = ALIGN(4);
        _sdata = .;
        *(.data)
        *(.data*)
        . = ALIGN(4);
        _edata = .;
    }} > RAM AT> FLASH

    .bss :
    {{
        . = ALIGN(4);
        _sbss = .;
        *(.bss)
        *(.bss*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = .;
    }} > RAM

    ._user_heap_stack :
    {{
        . = ALIGN(8);
        PROVIDE(end = .);
        PROVIDE(_end = .);
        . = . + 0x400;  /* Min heap */
        . = . + 0x400;  /* Min stack */
        . = ALIGN(8);
    }} > RAM
}}
"#,
        part = opts.mcu.part_number,
        define = opts.mcu.define,
        flash_kb = opts.mcu.flash_kb,
        ram_kb = opts.mcu.ram_kb,
        flash_origin = flash_origin,
        flash_size = flash_size,
    )
}

fn gen_system_c(opts: &GenerateOptions) -> String {
    format!(
        r#"/**
 * System initialization for {part} ({define})
 * Clock: {clock}MHz
 * Auto-generated by embtool
 */

#include <stdint.h>

#define {define}

uint32_t SystemCoreClock = {clock}000000UL;

void SystemInit(void) {{
    /* TODO: Configure system clocks
     * - Enable external oscillator if available
     * - Configure PLL for {clock}MHz operation
     * - Set flash wait states
     */
}}

void SystemCoreClockUpdate(void) {{
    /* TODO: Read actual clock configuration and update SystemCoreClock */
    SystemCoreClock = {clock}000000UL;
}}
"#,
        part = opts.mcu.part_number,
        define = opts.mcu.define,
        clock = opts.mcu.clock_mhz,
    )
}

fn gen_main_c(opts: &GenerateOptions) -> String {
    match opts.project_type {
        ProjectType::Bootloader => format!(
            r#"/**
 * Bootloader for {part}
 * Auto-generated by embtool
 */

#include <stdint.h>

#define APP_START_ADDR  0x00008000UL  /* Application starts at 32KB */

typedef void (*app_entry_t)(void);

static void jump_to_app(uint32_t addr) {{
    uint32_t app_sp = *(volatile uint32_t *)addr;
    uint32_t app_entry = *(volatile uint32_t *)(addr + 4);

    __asm volatile("MSR msp, %0" : : "r"(app_sp));
    ((app_entry_t)app_entry)();
}}

int main(void) {{
    /* TODO: Implement bootloader logic
     * - Check for firmware update request
     * - Validate application image
     * - Update firmware if needed
     */

    /* Jump to application */
    jump_to_app(APP_START_ADDR);

    /* Should never reach here */
    while (1) {{}}
}}
"#,
            part = opts.mcu.part_number,
        ),
        _ => format!(
            r#"/**
 * {name} — {part}
 * Auto-generated by embtool
 */

#include <stdint.h>

int main(void) {{
    /* TODO: Initialize peripherals */

    while (1) {{
        /* Main loop */
    }}
}}
"#,
            name = opts.name,
            part = opts.mcu.part_number,
        ),
    }
}

fn gen_clangd_config(opts: &GenerateOptions) -> String {
    generate_clangd_config(opts.mcu.define, opts.mcu.core, opts.mcu.fpu)
}

/// Generate .clangd config (used by both `new` and `init`)
pub fn generate_clangd_config(
    mcu_define: &str,
    core: &str,
    fpu: &str,
) -> String {
    let fpu_lines = match fpu {
        "hard" => "    - -mfloat-abi=hard\n    - -mfpu=fpv4-sp-d16",
        "softfp" => "    - -mfloat-abi=softfp\n    - -mfpu=fpv4-sp-d16",
        _ => "    - -mfloat-abi=soft",
    };

    format!(
        r#"# Auto-generated by embtool — clangd LSP configuration
# Enables code intelligence for ARM cross-compilation
#
# Requires: compile_commands.json in build/
# Generate with: embtool build (or cmake -B build -DCMAKE_EXPORT_COMPILE_COMMANDS=ON)

CompileFlags:
  CompilationDatabase: build/
  Add:
    - -D{define}
    - -mcpu={core}
    - -mthumb
{fpu}
    - --target=arm-none-eabi
  Remove:
    - -m*
    - --specs=*

Diagnostics:
  UnusedIncludes: Strict
  ClangTidy:
    Add:
      - bugprone-*
      - cert-*
      - performance-*
      - readability-*
    Remove:
      - modernize-use-trailing-return-type
      - readability-magic-numbers

InlayHints:
  Enabled: true
  ParameterNames: true
  DeducedTypes: true
"#,
        define = mcu_define,
        core = core,
        fpu = fpu_lines,
    )
}

/// Generate arm-toolchain.cmake (used by `new`, `init`, `setup`)
pub fn generate_toolchain_cmake(vendor: &str, version: &str) -> String {
    let tc_id = format!("{}-{}", vendor, version);
    format!(
        r#"# Auto-generated by embtool — DO NOT EDIT
# Toolchain: {VENDOR} ARM GCC {version}
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
        VENDOR = vendor.to_uppercase(),
        version = version,
        tc_id = tc_id,
    )
}

pub const CLANG_FORMAT_CONFIG: &str = r#"---
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

pub const CLANG_TIDY_CONFIG: &str = r#"---
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

pub const GITIGNORE: &str = r#"# Build output
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mcu_db;

    #[test]
    fn test_generate_application() {
        let dir = tempfile::tempdir().unwrap();
        let mcu = mcu_db::lookup("k64").unwrap();
        let opts = GenerateOptions {
            name: "test_app",
            mcu,
            project_type: ProjectType::Application,
            toolchain_vendor: "nxp",
            toolchain_version: "14.2.1",
            output_dir: dir.path(),
        };
        let files = generate_project(&opts).unwrap();
        assert!(files.contains(&PathBuf::from("embtool.toml")));
        assert!(files.contains(&PathBuf::from("CMakeLists.txt")));
        assert!(files.contains(&PathBuf::from("src/main.c")));
        assert!(files.contains(&PathBuf::from("system/startup.c")));
        assert!(files.contains(&PathBuf::from("system/linkerscript.ld")));

        // Verify embtool.toml is parseable
        let toml_content = std::fs::read_to_string(dir.path().join("embtool.toml")).unwrap();
        assert!(toml_content.contains("MK64F12"));
        assert!(toml_content.contains("nxp"));
    }

    #[test]
    fn test_generate_bootloader() {
        let dir = tempfile::tempdir().unwrap();
        let mcu = mcu_db::lookup("k64").unwrap();
        let opts = GenerateOptions {
            name: "test_boot",
            mcu,
            project_type: ProjectType::Bootloader,
            toolchain_vendor: "nxp",
            toolchain_version: "14.2.1",
            output_dir: dir.path(),
        };
        let files = generate_project(&opts).unwrap();

        // Bootloader linker script should have 32K flash
        let ld = std::fs::read_to_string(dir.path().join("system/linkerscript.ld")).unwrap();
        assert!(ld.contains("LENGTH = 32K"));

        // Main should have jump_to_app
        let main = std::fs::read_to_string(dir.path().join("src/main.c")).unwrap();
        assert!(main.contains("jump_to_app"));

        assert!(files.contains(&PathBuf::from("src/main.c")));
    }

    #[test]
    fn test_generate_library() {
        let dir = tempfile::tempdir().unwrap();
        let mcu = mcu_db::lookup("k64").unwrap();
        let opts = GenerateOptions {
            name: "test_lib",
            mcu,
            project_type: ProjectType::Library,
            toolchain_vendor: "nxp",
            toolchain_version: "14.2.1",
            output_dir: dir.path(),
        };
        let files = generate_project(&opts).unwrap();

        // Library should NOT have main.c, startup, or linkerscript
        assert!(!files.contains(&PathBuf::from("src/main.c")));
        assert!(!files.contains(&PathBuf::from("system/startup.c")));
        assert!(!files.contains(&PathBuf::from("system/linkerscript.ld")));

        // CMake should use add_library
        let cmake = std::fs::read_to_string(dir.path().join("CMakeLists.txt")).unwrap();
        assert!(cmake.contains("add_library"));
    }

    #[test]
    fn test_fpu_flags() {
        let dir = tempfile::tempdir().unwrap();
        let mcu = mcu_db::lookup("k66").unwrap(); // hard FPU
        let opts = GenerateOptions {
            name: "test_fpu",
            mcu,
            project_type: ProjectType::Application,
            toolchain_vendor: "nxp",
            toolchain_version: "14.2.1",
            output_dir: dir.path(),
        };
        generate_project(&opts).unwrap();
        let cmake = std::fs::read_to_string(dir.path().join("CMakeLists.txt")).unwrap();
        assert!(cmake.contains("-mfloat-abi=hard"));
        assert!(cmake.contains("-mfpu=fpv4-sp-d16"));
    }
}
