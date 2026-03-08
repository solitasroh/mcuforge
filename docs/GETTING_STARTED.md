# Getting Started with mcuforge

This guide will walk you through your first mcuforge project from installation to building firmware.

## Prerequisites

### System Requirements

- **OS**: Linux (x64) or Windows (x64)
- **Disk Space**: ~2GB for toolchain + tools
- **Network**: Internet connection for downloading toolchains

### Required Software

**Linux**:
```bash
# 7-Zip for toolchain extraction
sudo apt install p7zip-full

# Optional: for clangd LSP
sudo apt install clangd
```

**Windows**:
- [7-Zip](https://www.7-zip.org/download.html)

## Installation

### Option 1: Pre-built Binary (Recommended)

**Linux**:
```bash
# Download latest release
curl -LO https://github.com/solitasroh/mcuforge/releases/latest/download/mcuforge-v0.2.0-linux-x64

# Make executable
chmod +x mcuforge-v0.2.0-linux-x64

# Move to PATH
sudo mv mcuforge-v0.2.0-linux-x64 /usr/local/bin/mcuforge

# Verify installation
mcuforge --version
```

**Windows**:
1. Download `mcuforge-v0.2.0-windows-x64.exe` from [Releases](https://github.com/solitasroh/mcuforge/releases)
2. Rename to `mcuforge.exe`
3. Add to PATH or move to a directory in PATH
4. Verify: `mcuforge --version`

### Option 2: From Source

```bash
# Clone repository
git clone https://github.com/solitasroh/mcuforge.git
cd mcuforge

# Build release binary
cargo build --release

# Install
sudo cp target/release/mcuforge /usr/local/bin/

# Verify
mcuforge --version
```

## Your First Project

### Step 1: Create a New Project

```bash
mcuforge new blinky --mcu k64
```

This creates a project directory with:
- `embtool.toml` — Project configuration
- `CMakeLists.txt` — Build configuration
- `src/main.c` — Your application code
- `system/` — Startup and linker scripts
- `.clangd` — LSP configuration for IDE

### Step 2: Explore the Generated Code

```bash
cd blinky
cat src/main.c
```

You'll see a minimal application template:

```c
#include <stdint.h>
#include "MK64F12.h"

int main(void) {
    // TODO: Initialize peripherals
    
    while (1) {
        // Main loop
    }
    
    return 0;
}
```

### Step 3: Set Up the Environment

```bash
mcuforge setup
```

This will:
1. ✅ Check if NXP ARM GCC toolchain is installed (auto-install if missing)
2. ✅ Check if CMake is installed (auto-install if missing)
3. ✅ Generate `arm-toolchain.cmake`
4. ✅ Verify everything is ready

**First run** will download ~1GB of toolchain + tools. Subsequent runs are instant.

### Step 4: Build the Project

```bash
mcuforge build
```

Expected output:
```
🔨 Building blinky (Debug)
   MCU: MK64FN1M0VLL12 | Toolchain: nxp-14.2.1

   [Configure] ✅
   [Build]     ✅ (1.2s)

   Output: build/blinky.elf → .bin, .hex
   Flash:  148B / 1MB (0.0%)  ░░░░░░░░░░░░░░░░░░░░
   RAM:    2KB  / 256KB (0.8%) ░░░░░░░░░░░░░░░░░░░░
```

Artifacts:
- `build/blinky.elf` — ELF binary
- `build/blinky.bin` — Raw binary for flashing
- `build/blinky.hex` — Intel HEX format

### Step 5: Release Build

```bash
mcuforge build --release
```

Or shorthand:
```bash
mcuforge build -r
```

## Working with Different MCUs

### Available MCUs

```bash
mcuforge new project_name --mcu <id>
```

| MCU ID | Part Number | Flash | RAM | Frequency |
|--------|-------------|-------|-----|-----------|
| `k10d` | MK10DN512VLL10 | 512KB | 128KB | 100MHz |
| `k12` | MK12DN512VLH5 | 512KB | 128KB | 50MHz |
| `k22f` | MK22FX512VLL12 | 512KB | 128KB | 120MHz |
| `k64` | MK64FN1M0VLL12 | 1MB | 256KB | 120MHz |
| `k66` | MK66FN2M0VMD18 | 2MB | 256KB | 180MHz |

### Example: High-Performance Project

```bash
# MK66 with FPU and 2MB flash
mcuforge new high_perf --mcu k66
```

The generated linker script and startup code will automatically configure:
- Cortex-M4 with FPU (hard float)
- 2MB Flash @ 0x0000_0000
- 256KB RAM @ 0x2000_0000
- Vector table and clock initialization

## Bootloader Projects

### Creating a Bootloader

```bash
mcuforge new bootloader --mcu k64 --type bootloader
cd bootloader
```

Differences from application:
- **32KB flash** reserved for bootloader
- **Jump-to-app** logic at 0x8000
- Application region: 0x8000 - 0xFFFFF

### Bootloader Template

```c
#define APP_START_ADDR 0x00008000

static void jump_to_app(uint32_t addr) {
    uint32_t sp = *(volatile uint32_t *)addr;
    uint32_t pc = *(volatile uint32_t *)(addr + 4);
    
    __set_MSP(sp);
    void (*app_entry)(void) = (void (*)(void))pc;
    app_entry();
}

int main(void) {
    // Bootloader logic
    if (should_enter_app()) {
        jump_to_app(APP_START_ADDR);
    }
    
    // Firmware update logic
    while (1) {}
}
```

## Library Projects

### Creating a Library

```bash
mcuforge new mylib --mcu k64 --type library
cd mylib
```

Differences:
- No `src/main.c`
- No startup code or linker script
- `CMakeLists.txt` uses `add_library()` instead of `add_executable()`

Use for:
- Reusable driver libraries
- HAL abstractions
- Protocol implementations

## Code Quality Tools

### Formatting

Install clang-format:
```bash
mcuforge tool install clang-format 18
```

Format your code:
```bash
mcuforge format
```

Check formatting (CI mode):
```bash
mcuforge format --check
```

### Linting

Install clang-tidy:
```bash
mcuforge tool install clang-tidy 18
```

Lint your code:
```bash
mcuforge lint
```

Auto-fix issues:
```bash
mcuforge lint --fix
```

## IDE Integration

### VS Code

mcuforge auto-generates `.clangd` for LSP support.

**Install extensions**:
- `clangd` (LLVM)
- `CMake Tools` (Microsoft)

**Configure** (`.vscode/settings.json`):
```json
{
  "clangd.arguments": [
    "--compile-commands-dir=${workspaceFolder}/build",
    "--query-driver=/home/$USER/.embtool/toolchains/nxp-14.2.1/bin/arm-none-eabi-gcc"
  ],
  "cmake.configureArgs": [
    "-DCMAKE_TOOLCHAIN_FILE=${workspaceFolder}/arm-toolchain.cmake"
  ]
}
```

### CLion

1. Open project directory
2. CLion will auto-detect `CMakeLists.txt`
3. Set toolchain file: **Settings → Build → CMake → CMake options**:
   ```
   -DCMAKE_TOOLCHAIN_FILE=arm-toolchain.cmake
   ```

## Initializing Existing Projects

If you have an existing MCU project:

```bash
cd your-project
mcuforge init
```

Interactive prompts will ask:
1. Project name (default: directory name)
2. Target MCU
3. Project type
4. Toolchain version
5. CMake version
6. clang-format/tidy (optional)

**Result**: Generates `embtool.toml`, `arm-toolchain.cmake`, and `.clangd` without touching your source files.

## Troubleshooting

### Build Fails: Toolchain Not Found

```bash
# Re-run setup
mcuforge setup --force

# Or manually install
mcuforge toolchain install nxp:14.2
```

### CMake Not Found

```bash
# Check installation
mcuforge cmake list

# Install if missing
mcuforge cmake install 3.28
```

### Clean Build

```bash
rm -rf build
mcuforge build
```

### Verbose Output

```bash
mcuforge build --verbose
```

## Next Steps

- Read [SPECIFICATION.md](SPECIFICATION.md) for detailed architecture
- Explore [design docs](design/) for implementation details
- Check [GitHub Issues](https://github.com/solitasroh/mcuforge/issues) for known issues

## Getting Help

- **Documentation**: https://github.com/solitasroh/mcuforge
- **Issues**: https://github.com/solitasroh/mcuforge/issues
- **Email**: rsj0811@rootech.com

---

Happy forging! 🔥
