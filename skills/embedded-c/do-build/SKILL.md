---
name: do-build
description: "Builds firmware using CMake presets (Release/Debug/MinSizeRel), with optional clean build, and reports binary size and artifacts. Use when building firmware or after code changes. ALWAYS use this skill when the user mentions: 빌드, build, 컴파일, compile, 클린빌드, clean build, 펌웨어 빌드, cmake build, 빌드 확인, 컴파일 에러, 링크 에러, link error, 증분 빌드, incremental build, 프리셋 비교, preset comparison, .bin/.hex/.elf 생성, Flash 사용률 확인과 함께 빌드 요청, or wants to verify code changes compile successfully. Do NOT use for: ELF/map 파일 분석 without building (use check-binary-analysis), memory budget checks (use check-binary-analysis), static code analysis, code review, or test generation."
argument-hint: "[--preset=Debug|Release] [--clean]"
user-invokable: true
---

# MCU Build

1. Determine Build Type
   - Select preset: `Release`(default), `Debug`, `MinSizeRel`, `RelWithDebInfo`
   - Clean build required? (Default: No)

2. Clean (if requested)
   - Primary: `cmake --build --preset clean`
   - Fallback (if above fails): `rm -rf output/`

3. Configure Project
   - If `output/CMakeCache.txt` exists and preset matches: skip or let CMake fast-pass
   - Otherwise: `cmake --preset <Preset>`

4. Build Firmware
   - `cmake --build --preset <Preset> --parallel`

5. Report Status
   - Success:
     - `arm-none-eabi-size output/*.elf` (text/data/bss)
     - `ls -lh output/*.bin output/*.hex`
   - Failure — common patterns:
     - Linker `undefined reference`: missing source in CMakeLists.txt or unimplemented extern
     - `No such file or directory` for header: check include path in CMakeLists.txt
     - `region m_text overflowed`: code size exceeds 192KB Flash budget → try MinSizeRel or remove unused code
