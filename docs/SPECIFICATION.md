# embtool 상세 기획서

> **버전**: v0.1 Draft
> **작성일**: 2026-03-08
> **작성자**: 노수장 / 앨리스

---

## 1. 프로젝트 배경

### 1.1 현재 워크플로우의 문제점

```
현재 프로세스 (수동):
┌─────────────────────────────────────────────────────────┐
│ 1. NXP IDE 설치 → SDK 다운로드                           │
│ 2. SDK에서 ARM GCC 툴체인 추출                            │
│ 3. 툴체인을 사내 FTP 서버에 업로드                          │
│ 4. 프로젝트 디렉토리 수동 생성                              │
│ 5. CMakeLists.txt 수동 작성                               │
│ 6. arm-toolchain.cmake 수동 작성                          │
│ 7. CMake에서 FTP로 툴체인 다운로드 → 프로젝트 내부 배치       │
│ 8. 빌드 & 디버그 (PEMicro)                                │
└─────────────────────────────────────────────────────────┘

문제:
• 툴체인이 프로젝트 내부에 포함 → 프로젝트 크기 비대 (a2750mcu: 187MB)
• 프로젝트마다 동일 툴체인 중복 저장
• arm-toolchain.cmake를 매번 수동 작성 (경로 하드코딩)
• 새 프로젝트 생성 시 보일러플레이트 작업 반복
• 팀원 환경 통일 어려움
• 윈도우/리눅스 경로 분기 수동 관리
```

### 1.2 기존 프로젝트 구조 분석

**a2750mcu (모노레포, 187MB)**
```
a2750mcu/
├── products/
│   ├── a2750lm_application/      # MK64F12, cortex-m4
│   │   ├── CMakeLists.txt
│   │   ├── arm-toolchain.cmake   # 경로 하드코딩 ⚠️
│   │   ├── System/
│   │   │   └── linkerscript.ld
│   │   └── Sources/
│   ├── a2750lm_bootloader/       # 같은 MCU, 같은 툴체인 중복
│   ├── a2750io_application/
│   ├── a2750p_application/
│   ├── a2700dsp/
│   └── ... (15+ 프로젝트)
├── components/                   # 공유 라이브러리
└── mcu-dev/                      # Claude Code skills
```

**a2550mcu (레거시, .cproject 기반)**
```
a2550mcu/
├── products/
│   ├── a2550io/                  # MK10D7 (KDS IDE 기반)
│   │   ├── .cproject             # Eclipse 프로젝트 ⚠️
│   │   ├── Project_Settings/
│   │   └── Sources/
│   └── ... (20+ 프로젝트)
├── components/
└── tools/
```

### 1.3 사용 중인 MCU 목록

| MCU | 코어 | 시리즈 | 프로젝트 |
|-----|------|--------|---------|
| MK10DN512 (K10D) | Cortex-M4 | Kinetis K10 | a2550io 등 |
| MK12D5 | Cortex-M4 | Kinetis K12 | a2550 일부 |
| MK22FX512 (K22F) | Cortex-M4 | Kinetis K22 | ct3p 등 |
| MK64FN1M0 (K64) | Cortex-M4 | Kinetis K64 | a2750lm 등 |
| MK66FN2M0 (K66) | Cortex-M4 | Kinetis K66 | (예정) |
| STM32 (TBD) | Cortex-M? | STM32 | (선정 중) |

---

## 2. embtool 목표

### 2.1 핵심 목표
```
embtool이 해결하는 것:
┌─────────────────────────────────────────────────────────┐
│ • 툴체인을 시스템 레벨에서 관리 (프로젝트 밖)               │
│ • 한 줄 명령으로 프로젝트 생성 (보일러플레이트 제거)          │
│ • arm-toolchain.cmake 자동 생성                          │
│ • 팀원 환경 자동 통일                                      │
│ • 크로스플랫폼 (Linux/Windows)                            │
└─────────────────────────────────────────────────────────┘
```

### 2.2 비전
```
AS-IS:                          TO-BE:
IDE 설치 → SDK 추출             embtool toolchain install 13.3
FTP 업로드                       (자동 다운로드 & 관리)
수동 디렉토리 생성               embtool new my-project --mcu k64
CMake 수동 작성                  (자동 생성)
경로 하드코딩                    embtool build
187MB 프로젝트                   경량 프로젝트 (~1MB 소스만)
```

---

## 3. 기능 설계

### 3.1 Phase 1: 툴체인 관리 (MVP)

#### `embtool toolchain install <version>`
```bash
# ARM GNU Toolchain 설치 (developer.arm.com에서 자동 다운로드)
$ embtool toolchain install 13.3
📦 Downloading ARM GNU Toolchain 13.3.rel1...
   [████████████████████████████████] 100% (245 MB)
📂 Installing to ~/.embtool/toolchains/13.3.rel1/
✅ ARM GNU Toolchain 13.3.rel1 installed successfully
   arm-none-eabi-gcc 13.3.1

# 특정 버전 설치
$ embtool toolchain install 12.2
$ embtool toolchain install 10.3
```

#### `embtool toolchain list`
```bash
$ embtool toolchain list
Installed toolchains:
  * 13.3.rel1  (active)
    12.2.rel1
    10.3-2021.10

System paths:
  /opt/Freescale/KDS_v3/toolchain  (detected, legacy)
```

#### `embtool toolchain use <version>`
```bash
$ embtool toolchain use 12.2
🔄 Switched to ARM GNU Toolchain 12.2.rel1
   arm-none-eabi-gcc 12.2.1
```

#### `embtool toolchain remove <version>`
```bash
$ embtool toolchain remove 10.3
🗑️  Removed ARM GNU Toolchain 10.3-2021.10
   Freed 512 MB
```

#### 저장 구조
```
~/.embtool/
├── config.toml                    # 전역 설정
├── toolchains/
│   ├── 13.3.rel1/
│   │   └── arm-none-eabi/
│   │       ├── bin/
│   │       ├── lib/
│   │       └── include/
│   └── 12.2.rel1/
│       └── arm-none-eabi/
└── cache/                         # 다운로드 캐시
    └── gcc-arm-none-eabi-13.3-...tar.xz
```

#### 전역 설정 (config.toml)
```toml
[toolchain]
default = "13.3.rel1"
install_dir = "~/.embtool/toolchains"

[toolchain.sources]
# ARM 공식 다운로드 URL 패턴
arm_gnu = "https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads"

# 사내 FTP 미러 (선택)
[toolchain.mirror]
enabled = true
url = "ftp://internal-server/toolchains/"

[debug]
default_probe = "pemicro"
```

---

### 3.2 Phase 2: 프로젝트 생성

#### `embtool new <name> --mcu <mcu>`
```bash
$ embtool new a2750lm_application --mcu k64

🆕 Creating project 'a2750lm_application'
   MCU: MK64FN1M0VLL12 (Cortex-M4, 1MB Flash, 256KB RAM)
   Toolchain: 13.3.rel1 (from ~/.embtool)

📁 Project structure:
   a2750lm_application/
   ├── CMakeLists.txt           # 자동 생성
   ├── arm-toolchain.cmake      # embtool 경로 자동 참조
   ├── embtool.toml              # 프로젝트 설정
   ├── system/
   │   ├── startup.c
   │   ├── linkerscript.ld
   │   └── system_MK64F12.c
   └── src/
       └── main.c

✅ Project created! Next steps:
   cd a2750lm_application
   embtool build
```

#### 프로젝트 설정 (embtool.toml)
```toml
[project]
name = "a2750lm_application"
version = "1.0.0"

[target]
mcu = "MK64FN1M0VLL12"
core = "cortex-m4"
fpu = "soft"                     # soft | hard | softfp
flash = "1M"
ram = "256K"

[toolchain]
version = "13.3.rel1"            # 프로젝트 고정 버전

[build]
c_standard = "c99"
optimization.debug = "O0"
optimization.release = "O1"
linker_script = "system/linkerscript.ld"

[build.defines]
target = ["MK64F12"]
custom = []

[build.flags]
common = [
    "-ffunction-sections",
    "-fno-common",
    "-fdata-sections",
    "-fmessage-length=0",
    "-fsigned-char",
]

[debug]
probe = "pemicro"
interface = "swd"

[components]
# 외부 컴포넌트 참조 (모노레포의 components/ 대체)
# path = ["../../components/modbus", "../../components/hal"]
```

#### 자동 생성되는 arm-toolchain.cmake
```cmake
# Auto-generated by embtool - DO NOT EDIT
# Toolchain: ARM GNU 13.3.rel1
# Run 'embtool toolchain regenerate' to update

cmake_minimum_required(VERSION 3.20)

# embtool managed toolchain path
set(EMBTOOL_HOME "$ENV{HOME}/.embtool")
set(EMBTOOL_TOOLCHAIN_VERSION "13.3.rel1")
set(ARM_TOOLCHAIN_ROOT "${EMBTOOL_HOME}/toolchains/${EMBTOOL_TOOLCHAIN_VERSION}")
set(ARM_TOOLCHAIN_PATH "${ARM_TOOLCHAIN_ROOT}/bin/")

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR arm)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_C_COMPILER "${ARM_TOOLCHAIN_PATH}arm-none-eabi-gcc")
set(CMAKE_CXX_COMPILER "${ARM_TOOLCHAIN_PATH}arm-none-eabi-g++")
set(CMAKE_ASM_COMPILER "${ARM_TOOLCHAIN_PATH}arm-none-eabi-gcc")
set(CMAKE_AR "${ARM_TOOLCHAIN_PATH}arm-none-eabi-ar")
set(CMAKE_OBJCOPY "${ARM_TOOLCHAIN_PATH}arm-none-eabi-objcopy" CACHE INTERNAL "")
set(CMAKE_SIZE "${ARM_TOOLCHAIN_PATH}arm-none-eabi-size" CACHE INTERNAL "")

set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)
```

---

### 3.3 Phase 3: 빌드 시스템

#### `embtool build`
```bash
$ embtool build
🔨 Building a2750lm_application (Debug)
   MCU: MK64FN1M0VLL12
   Toolchain: ARM GNU 13.3.rel1
   
   [CMake Configure] ✅
   [CMake Build]     ████████████████ 100%
   
✅ Build successful
   Output: build/a2750lm_application.elf
   Flash:  45,312 / 1,048,576 bytes (4.3%)
   RAM:    12,288 / 262,144 bytes (4.7%)

$ embtool build --release
🔨 Building a2750lm_application (Release)
   ...
```

#### `embtool flash`
```bash
$ embtool flash
🔌 Detecting probe... PEMicro Multilink (USB1)
📤 Flashing a2750lm_application.elf
   [████████████████████████████████] 100%
✅ Flash complete (45,312 bytes)
```

---

### 3.4 Phase 4: 프로젝트 마이그레이션

기존 프로젝트를 embtool 관리로 전환:

#### `embtool migrate`
```bash
$ cd ~/works/a2750mcu/products/a2750lm_application
$ embtool migrate

🔍 Analyzing existing project...
   Found: CMakeLists.txt
   Found: arm-toolchain.cmake
   Detected MCU: MK64F12 (from -DMK64F12)
   Detected Core: cortex-m4
   Detected FPU: soft
   Detected Toolchain: KDS_v3 (legacy path)

📋 Migration plan:
   1. Generate embtool.toml from existing CMakeLists.txt
   2. Replace arm-toolchain.cmake with embtool-managed version
   3. Install matching toolchain (if needed)
   4. Verify build output matches

Proceed? [Y/n] y

✅ Migration complete!
   embtool.toml created
   arm-toolchain.cmake updated (backup: arm-toolchain.cmake.bak)
```

---

## 4. MCU 데이터베이스

### 4.1 지원 MCU 정의

embtool 내부에 MCU 메타데이터 내장:

```rust
// src/mcu/nxp.rs
pub struct McuInfo {
    pub id: &'static str,           // "k64"
    pub part_number: &'static str,  // "MK64FN1M0VLL12"
    pub define: &'static str,       // "MK64F12"
    pub core: &'static str,         // "cortex-m4"
    pub fpu: &'static str,          // "soft"
    pub flash_kb: u32,              // 1024
    pub ram_kb: u32,                // 256
    pub family: &'static str,      // "kinetis"
    pub series: &'static str,      // "K64"
}
```

| 별칭 | Part Number | Define | Flash | RAM |
|------|-------------|--------|-------|-----|
| `k10d` | MK10DN512VLL10 | MK10D7 | 512KB | 128KB |
| `k12` | MK12DN512VLH5 | MK12D5 | 512KB | 128KB |
| `k22f` | MK22FX512VLL12 | MK22F12 | 512KB | 128KB |
| `k64` | MK64FN1M0VLL12 | MK64F12 | 1MB | 256KB |
| `k66` | MK66FN2M0VMD18 | MK66F18 | 2MB | 256KB |

---

## 5. 프로젝트 템플릿

### 5.1 기본 템플릿 구조

```
templates/
├── nxp-kinetis/
│   ├── CMakeLists.txt.hbs         # Handlebars 템플릿
│   ├── arm-toolchain.cmake.hbs
│   ├── embtool.toml.hbs
│   ├── system/
│   │   ├── startup_{{mcu}}.c
│   │   ├── system_{{mcu}}.c
│   │   └── linkerscript.ld.hbs
│   └── src/
│       └── main.c
└── stm32/                         # Phase 2
    └── ...
```

### 5.2 프로젝트 유형

```bash
embtool new my-app --mcu k64                    # 기본 application
embtool new my-boot --mcu k64 --type bootloader  # 부트로더
embtool new my-lib --mcu k64 --type library      # 라이브러리/컴포넌트
```

---

## 6. 컴포넌트 관리 (Phase 5, 향후)

기존 모노레포의 `components/` 디렉토리를 독립 패키지로 관리:

```bash
# 컴포넌트 추가
embtool add modbus --path ../../components/modbus
embtool add hal-k64 --git https://github.com/company/hal-k64

# embtool.toml
[dependencies]
modbus = { path = "../../components/modbus" }
hal-k64 = { git = "https://github.com/company/hal-k64", tag = "v1.0" }
```

---

## 7. 아키텍처

### 7.1 모듈 구조

```
src/
├── main.rs                    # CLI entry point (clap)
├── commands/
│   ├── mod.rs
│   ├── toolchain.rs           # toolchain install/list/use/remove
│   ├── new.rs                 # project scaffolding
│   ├── build.rs               # cmake wrapper
│   ├── flash.rs               # PEMicro flash
│   ├── config.rs              # configuration management
│   └── migrate.rs             # legacy project migration
├── core/
│   ├── mod.rs
│   ├── toolchain_manager.rs   # 툴체인 다운로드/설치/전환
│   ├── project.rs             # embtool.toml 파싱/관리
│   ├── mcu_db.rs              # MCU 메타데이터
│   └── template.rs            # 프로젝트 템플릿 엔진
├── mcu/
│   ├── mod.rs
│   ├── nxp.rs                 # NXP Kinetis MCU 정의
│   └── stm32.rs               # STM32 MCU 정의 (향후)
└── utils/
    ├── mod.rs
    ├── download.rs            # HTTP/FTP 다운로드 + 진행바
    ├── archive.rs             # tar.xz 해제
    └── paths.rs               # 경로 관리 (~/.embtool)
```

### 7.2 의존성

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }     # CLI 파싱
serde = { version = "1", features = ["derive"] }     # 직렬화
toml = "0.8"                                         # 설정 파일
reqwest = { version = "0.12", features = ["blocking", "rustls-tls"] }  # HTTP
indicatif = "0.17"                                   # 진행바
dialoguer = "0.11"                                   # 대화형 입력
colored = "3"                                        # 터미널 색상
dirs = "6"                                           # 시스템 디렉토리
anyhow = "1"                                         # 에러 처리
handlebars = "6"                                     # 템플릿 엔진
flate2 = "1"                                         # gzip
tar = "0.4"                                          # tar 아카이브
xz2 = "0.1"                                          # xz 해제
```

---

## 8. 구현 로드맵

### Phase 1: 툴체인 관리 (Week 1-2)
- [ ] `~/.embtool` 디렉토리 구조 생성
- [ ] `config.toml` 관리
- [ ] ARM GNU Toolchain 다운로드 URL 파싱
- [ ] 다운로드 + 진행바
- [ ] tar.xz 해제 및 설치
- [ ] `toolchain install/list/use/remove` 구현
- [ ] 버전 전환 (symlink 방식)

### Phase 2: 프로젝트 생성 (Week 3-4)
- [ ] MCU 데이터베이스 구축
- [ ] 프로젝트 템플릿 시스템
- [ ] `embtool new --mcu` 구현
- [ ] CMakeLists.txt 자동 생성
- [ ] arm-toolchain.cmake 자동 생성
- [ ] embtool.toml 생성
- [ ] 스타트업/링커스크립트 번들

### Phase 3: 빌드 & 플래시 (Week 5-6)
- [ ] CMake 래핑 (`embtool build`)
- [ ] 빌드 결과 요약 (Flash/RAM 사용량)
- [ ] PEMicro 연동 (`embtool flash`)
- [ ] 빌드 프로파일 (Debug/Release)

### Phase 4: 마이그레이션 (Week 7-8)
- [ ] 기존 CMakeLists.txt 파싱
- [ ] arm-toolchain.cmake 분석
- [ ] 자동 embtool.toml 생성
- [ ] 마이그레이션 검증 (빌드 결과 비교)

### Phase 5: 고급 기능 (향후)
- [ ] 컴포넌트/패키지 관리
- [ ] STM32 지원
- [ ] 사내 FTP 미러 연동
- [ ] CI/CD 지원 (GitHub Actions)
- [ ] VS Code 확장

---

## 9. 차별화 포인트

| 기존 도구 | embtool |
|-----------|---------|
| IDE 종속 (MCUXpresso, KDS) | IDE 독립, CLI 기반 |
| 툴체인이 프로젝트 내부 | 시스템 레벨 관리 (nvm 방식) |
| 수동 CMake 작성 | 자동 생성 |
| 벤더별 별도 도구 | NXP + STM32 통합 |
| Windows 중심 | 크로스플랫폼 |
| 프로젝트 187MB | 소스만 ~1MB |

---

*기획서 v0.1 - 2026-03-08*
