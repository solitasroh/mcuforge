# mcuforge 🔥

> Cortex-M MCU development toolkit — ARM GCC, CMake, and build system manager

**mcuforge** simplifies embedded firmware development for Cortex-M MCUs by managing your entire toolchain, build system, and project scaffolding in one unified CLI.

[![Release](https://img.shields.io/github/v/release/solitasroh/mcuforge)](https://github.com/solitasroh/mcuforge/releases)
[![Tests](https://img.shields.io/badge/tests-46%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## ✨ Features

- **📦 Toolchain Management** — Download and manage ARM GCC toolchains from Cloudflare R2
- **🔧 Dev Tools** — CMake, clang-format, and clang-tidy version management
- **🚀 Project Scaffolding** — Generate complete projects with `mcuforge new` or initialize existing ones with `mcuforge init`
- **🏗️ Build System** — Integrated CMake builds with Flash/RAM usage reports
- **✨ Code Quality** — Format with clang-format and lint with clang-tidy
- **📊 Modern TUI** — Beautiful terminal UI with progress bars and status reports
- **🔍 LSP Ready** — Auto-generates `.clangd` config for IDE integration

## 🎯 Supported MCUs

Currently supports **NXP Kinetis** Cortex-M4 MCUs:

| ID | MCU | Core | Clock | Flash | RAM |
|----|-----|------|-------|-------|-----|
| `k10d` | MK10DN512VLL10 | Cortex-M4 | 100 MHz | 512 KB | 128 KB |
| `k12` | MK12DN512VLH5 | Cortex-M4 | 50 MHz | 512 KB | 128 KB |
| `k22f` | MK22FX512VLL12 | Cortex-M4 | 120 MHz | 512 KB | 128 KB |
| `k64` | MK64FN1M0VLL12 | Cortex-M4 | 120 MHz | 1 MB | 256 KB |
| `k66` | MK66FN2M0VMD18 | Cortex-M4 | 180 MHz | 2 MB | 256 KB |

*STM32 support coming soon.*

## 🚀 Quick Start

```bash
# Create a new project
mcuforge new my_project --mcu k64

# Enter project directory
cd my_project

# Install all required tools (toolchain, cmake, etc.)
mcuforge setup

# Build
mcuforge build

# Format code
mcuforge format

# Lint code
mcuforge lint
```

## 📥 Installation

### Pre-built Binaries

Download from [Releases](https://github.com/solitasroh/mcuforge/releases):

**Linux (x64)**:
```bash
curl -LO https://github.com/solitasroh/mcuforge/releases/latest/download/mcuforge-v0.2.0-linux-x64
chmod +x mcuforge-v0.2.0-linux-x64
sudo mv mcuforge-v0.2.0-linux-x64 /usr/local/bin/mcuforge
```

**Windows (x64)**:
```powershell
# Download mcuforge-v0.2.0-windows-x64.exe from releases
# Add to PATH
```

### From Source

**Requirements**: Rust 1.94+ (edition 2024)

```bash
git clone https://github.com/solitasroh/mcuforge.git
cd mcuforge
cargo build --release
sudo cp target/release/mcuforge /usr/local/bin/
```

### Additional Dependencies

**Linux**:
```bash
# 7-Zip (for toolchain extraction)
sudo apt install p7zip-full
```

**Windows**: Install [7-Zip](https://www.7-zip.org/)

## 📚 Usage

### Project Types

mcuforge supports three project types:

- **application** (default) — Standard MCU application with `main()`
- **bootloader** — Bootloader with 32KB flash and jump-to-app logic
- **library** — Static library without startup code

```bash
mcuforge new bootloader --mcu k66 --type bootloader
mcuforge new mylib --mcu k22f --type library
```

### Toolchain Management

```bash
# List installed toolchains
mcuforge toolchain list

# List available versions
mcuforge toolchain list --available

# Install specific version
mcuforge toolchain install nxp:14.2

# Switch active toolchain
mcuforge toolchain use nxp:14.2
```

### Build Profiles

```bash
# Debug build (default)
mcuforge build

# Release build
mcuforge build --release
# or shorthand
mcuforge build -r

# Clean build
mcuforge build --clean

# Verbose output
mcuforge build --verbose
```

### Development Tools

```bash
# Install CMake
mcuforge cmake install 3.28

# Install code quality tools
mcuforge tool install clang-format 18
mcuforge tool install clang-tidy 18

# List installed tools
mcuforge tool list
```

### Initialize Existing Project

```bash
cd your-existing-project
mcuforge init  # Interactive prompts
```

## 🏗️ Project Structure

```
my_project/
├── embtool.toml          # Project configuration
├── CMakeLists.txt        # Build configuration
├── arm-toolchain.cmake   # ARM GCC toolchain setup
├── .clangd               # LSP configuration
├── .gitignore
├── src/
│   └── main.c
└── system/
    ├── startup.c         # Startup code
    ├── linkerscript.ld   # Memory layout
    └── system_*.c        # System initialization
```

## ⚙️ Configuration

`embtool.toml` example:

```toml
[project]
name = "my_project"
type = "application"

[target]
mcu = "MK64F12"
core = "cortex-m4"
fpu = "soft"
flash = "1MB"
ram = "256KB"

[toolchain]
vendor = "nxp"
version = "14.2.1"

[cmake]
version = "3.28"

[tools.clang-format]
version = "18"

[tools.clang-tidy]
version = "18"
```

## 📖 Commands Reference

| Command | Description |
|---------|-------------|
| `mcuforge new <name>` | Create new project |
| `mcuforge init` | Initialize existing project |
| `mcuforge setup` | Install all required tools |
| `mcuforge build` | Build project |
| `mcuforge format` | Format code |
| `mcuforge lint` | Lint code |
| `mcuforge config` | Show configuration |
| `mcuforge toolchain` | Manage ARM GCC toolchains |
| `mcuforge cmake` | Manage CMake versions |
| `mcuforge tool` | Manage dev tools |

See `mcuforge <command> --help` for detailed usage.

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*'
```

**Current status**: 46/46 tests passing ✅

## 🔧 Development

### Project Structure

```
src/
├── commands/    # CLI command handlers
├── core/        # Business logic
│   ├── builder.rs           # CMake build wrapper
│   ├── cmake_provider.rs    # CMake management
│   ├── clang_provider.rs    # clang-format/tidy
│   ├── mcu_db.rs            # MCU database
│   └── template.rs          # Project scaffolding
├── mcu/         # MCU definitions
├── ui/          # TUI components (iocraft)
└── utils/       # Common utilities
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Windows cross-compile (from Linux)
cargo build --release --target x86_64-pc-windows-gnu
```

### Design Documents

See [docs/design/](docs/design/) for detailed architecture and phase-by-phase design documents.

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **ARM GCC** toolchains from NXP
- **CMake** from Kitware
- **clang-format/clang-tidy** static binaries from [cpp-linter/clang-tools-static-binaries](https://github.com/cpp-linter/clang-tools-static-binaries)
- **iocraft** for beautiful TUI components

---

**mcuforge** — Forge your MCU firmware with confidence 🔥
