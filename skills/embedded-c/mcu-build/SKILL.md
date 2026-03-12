---
name: mcu-build
description: "Builds firmware using CMake presets (Release/Debug/MinSizeRel), with optional clean build, and reports binary size and artifacts. Use when building firmware or after code changes. Do NOT use for non-CMake projects."
argument-hint: "[--preset=Debug|Release] [--clean]"
user-invokable: true
---

# MCU Build

1. Determine Build Type
   - Ask user to select build type: `Release`, `Debug`, `MinSizeRel`, `RelWithDebInfo` (Default: `Release`)
   - Ask if `Clean` build is required (Default: `No`)

2. Configure Project
   - If Clean build selected:
     - Run `cmake --build --preset <BuildType> --target clean` or remove `output/` directory manually if preferred.
   - Run `cmake --preset <BuildType>`

3. Build Firmware
   - Run `cmake --build --preset <BuildType> --parallel`

4. Report Status
   - If build succeeds:
     - Show size: `arm-none-eabi-size output/*.elf`
     - List artifacts: `ls -lh output/*.bin output/*.hex`
   - If build fails:
     - Capture error output
     - Suggest potential fixes based on error messages or general CMake knowledge.
