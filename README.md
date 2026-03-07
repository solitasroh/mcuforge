# embtool 🔧

Embedded toolchain manager for NXP and STM32 MCUs.

## Features (Planned)

- **Toolchain Management**: Install, switch, and manage ARM GCC toolchain versions
- **Project Scaffolding**: Generate project templates for supported MCUs
- **SDK Management**: Download and manage NXP MCUXpresso SDK / STM32 HAL
- **Build System**: CMake-based build configuration

## Supported MCUs

### NXP Kinetis
- MK10DN512 (K10D)
- MK22FX512 (K22F)
- MK64FN1M0 (K64)
- MK66FN2M0 (K66)

### STM32 (Coming Soon)
- TBD

## Installation

```bash
cargo install embtool
```

## Usage

```bash
# Install ARM GCC toolchain
embtool toolchain install 13.3

# List installed toolchains
embtool toolchain list

# Create a new project
embtool new my-project --mcu k22f

# Build project
embtool build
```

## License

MIT
