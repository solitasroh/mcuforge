# embtool 상세 기획서

> **버전**: v0.3 Draft
> **작성일**: 2026-03-08
> **작성자**: 노수장

---

## 1. 프로젝트 배경

### 1.1 현재 워크플로우의 문제점

```
현재 프로세스 (수동):
┌─────────────────────────────────────────────────────────┐
│ 1. NXP IDE 설치 → SDK 다운로드                           │
│ 2. SDK에서 ARM GCC 툴체인 추출                            │
│ 3. 툴체인을 사내 서버에 업로드                              │
│ 4. 프로젝트 디렉토리 수동 생성                              │
│ 5. CMakeLists.txt 수동 작성                               │
│ 6. arm-toolchain.cmake 수동 작성                          │
│ 7. CMake에서 툴체인 다운로드 → 프로젝트 내부 배치              │
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

### 1.4 팀 환경

```
팀 구성:
• 인원: 4-5명
• 개발 OS: Windows (MCU 개발)
• CI/CD: GitLab CI 또는 Jenkins (Linux 컨테이너/에이전트)
• 네트워크: 외부 인터넷 접근 가능
• 현재 배포: 사내 서버 → CMake 스텝에서 자동 다운로드
```

**팀 워크플로우 요구사항:**
```
팀원 온보딩 (현재):                  팀원 온보딩 (embtool):
┌────────────────────────┐          ┌────────────────────────┐
│ 1. IDE 설치 (1시간)     │          │ 1. embtool.exe 설치    │
│ 2. SDK 다운로드 (30분)  │          │ 2. git clone project   │
│ 3. 서버 경로 확인      │          │ 3. embtool setup       │
│ 4. CMake 환경 설정     │          │    → 끝! (5분)         │
│ 5. 빌드 테스트/디버그   │          └────────────────────────┘
│ 6. 경로 안 맞으면 삽질  │
│    (30분~반나절)        │
└────────────────────────┘
```

**CI/CD 요구사항:**
```yaml
# GitLab CI 예시 (목표)
build:
  image: rust-embedded  # 또는 ubuntu + embtool 설치
  script:
    - embtool setup          # 툴체인 자동 설치
    - embtool build --release
  artifacts:
    paths:
      - build/*.elf
      - build/*.bin

# Jenkins 예시 (목표)
pipeline {
  stages {
    stage('Setup') {
      steps { sh 'embtool setup' }
    }
    stage('Build') {
      steps { sh 'embtool build --release' }
    }
  }
}
```

---

## 2. embtool 목표

### 2.1 핵심 목표
```
embtool이 해결하는 것:
┌─────────────────────────────────────────────────────────┐
│ • 툴체인을 시스템 레벨에서 관리 (프로젝트 밖)               │
│ • 한 줄 명령으로 프로젝트 생성 (보일러플레이트 제거)          │
│ • arm-toolchain.cmake 자동 생성                          │
│ • embtool setup 한 줄로 팀원 환경 자동 통일                │
│ • 크로스플랫폼 (Windows 개발 + Linux CI/CD)               │
│ • embtool.toml → Git 커밋 → 팀 전원 동일 환경 보장         │
└─────────────────────────────────────────────────────────┘
```

### 2.2 비전
```
AS-IS:                          TO-BE:
IDE 설치 → SDK 추출             embtool toolchain install 13.3
수동 업로드                       (자동 다운로드 & 관리)
수동 디렉토리 생성               embtool new my-project --mcu k64
CMake 수동 작성                  (자동 생성)
경로 하드코딩                    embtool build
187MB 프로젝트                   경량 프로젝트 (~1MB 소스만)
```

---

## 3. 기능 설계

### 3.0 Phase 0: 팀 온보딩 (setup)

모든 기능의 전제 — `embtool setup`으로 팀원이 즉시 개발 시작 가능:

#### `embtool setup`
```bash
# 팀원이 프로젝트를 clone한 후 실행
$ git clone https://gitlab.company.com/firmware/a2750mcu.git
$ cd a2750mcu/products/a2750lm_application
$ embtool setup

🔍 Reading embtool.toml...
   Project: a2750lm_application
   MCU: MK64FN1M0VLL12 (Cortex-M4)
   Required toolchain: nxp-14.2.1

📦 Toolchain nxp-14.2.1 not found locally.
   Downloading nxp-14.2.1-win-x64.7z (118 MB)
   [████████████████████████████████] 100% (118/118 MB)
   Verifying SHA256... ✅
   Installing to C:\Users\{user}\.embtool\toolchains\nxp-14.2.1\

🔧 Generating arm-toolchain.cmake...
✅ Setup complete! Run 'embtool build' to build.
```

**핵심 원리:**
```
Git에 커밋되는 것:              로컬에만 존재하는 것:
┌────────────────────┐        ┌─────────────────────────┐
│ embtool.toml       │        │ ~/.embtool/toolchains/  │
│ (툴체인 버전 고정)  │   →    │ (실제 바이너리)          │
│                    │        │                         │
│ arm-toolchain.cmake│        │ build/                  │
│ (자동 생성, 경로    │        │ (빌드 산출물)            │
│  ~/.embtool 참조)  │        └─────────────────────────┘
└────────────────────┘
  팀원 모두 동일              OS별 자동 해석
```

#### 팀 .gitignore 패턴
```gitignore
# embtool
build/
# arm-toolchain.cmake는 커밋 (embtool이 관리하지만 IDE 호환용)
```

#### Windows 경로 처리
```cmake
# arm-toolchain.cmake (자동 생성, OS 자동 감지)
if(WIN32)
    set(EMBTOOL_HOME "$ENV{USERPROFILE}/.embtool")
else()
    set(EMBTOOL_HOME "$ENV{HOME}/.embtool")
endif()
set(ARM_TOOLCHAIN_ROOT "${EMBTOOL_HOME}/toolchains/nxp-14.2.1")
```

#### CI/CD 모드
```bash
# CI 환경에서는 --ci 플래그로 대화형 프롬프트 비활성화
$ embtool setup --ci

# 또는 환경변수로 제어
$ EMBTOOL_CI=1 embtool setup
```

#### embtool 배포 방법
```
팀원 설치 방법:
1. GitHub Releases에서 OS별 바이너리 다운로드
   - embtool-windows-x86_64.exe → C:\tools\embtool.exe
   - embtool-linux-x86_64      → /usr/local/bin/embtool
2. PATH에 추가
3. 끝!

CI/CD 설치 (Dockerfile/스크립트):
   curl -L https://github.com/solitasroh/embtool/releases/latest/download/embtool-linux-x86_64 -o /usr/local/bin/embtool
   chmod +x /usr/local/bin/embtool

사내 배포 (대안):
   사내 공유 폴더에 embtool 바이너리도 같이 올려서 배포
```

---

### 3.1 Phase 1: 툴체인 관리 (MVP)

#### 툴체인 배포 아키텍처

```
embtool-toolchains (GitHub, 메타데이터 리포):
  https://github.com/solitasroh/embtool-toolchains
  └── versions.json          ← 버전/해시/URL 메타데이터

Cloudflare R2 (바이너리 저장소, 인증 불필요):
  https://pub-25d9755030a54c3280b7a9f68e9bf67c.r2.dev/
  ├── versions.json
  ├── nxp-14.2.1-linux-x64.7z
  ├── nxp-14.2.1-win-x64.7z
  ├── nxp-13.2.1-win-x64.7z
  └── stm-13.3.1-win-x64.7z

사내 NAS 미러 (선택, 우선순위 높음):
  \\nas\share\embtool\toolchains\
  └── (R2와 동일 구조)
```

**다운로드 우선순위:**
```
1. mirror.enabled → NAS/사내 서버에서 (인증X, 최고속)
2. mirror 실패 + fallback → R2에서 다운로드
3. mirror.enabled=false → R2 직접 (기본)
```

#### 벤더별 툴체인

| 벤더 | 버전 | GCC | 특이사항 | Win | Linux |
|------|------|-----|---------|-----|-------|
| **NXP** | 14.2.1 | 14.2.1 | redlib, NXP features, newlib-nano | ✅ | ✅ |
| **NXP** | 13.2.1 | 13.2.1 | redlib, NXP features, newlib-nano | ✅ | - |
| **STM** | 13.3.1 | 13.3.1 | newlib-nano (순수 ARM GCC) | ✅ | - |

#### `embtool toolchain install <vendor:version>`
```bash
# NXP 툴체인 설치
$ embtool toolchain install nxp:14.2
📦 Installing NXP ARM GCC 14.2.1...
   Downloading nxp-14.2.1-win-x64.7z (118 MB)
   [████████████████████████████████] 100% (118/118 MB)
   Verifying SHA256... ✅
📂 Installing to ~/.embtool/toolchains/nxp-14.2.1/
✅ NXP ARM GCC 14.2.1 installed
   arm-none-eabi-gcc (Arm GNU Toolchain 14.2.Rel1) 14.2.1

# STM 툴체인 설치
$ embtool toolchain install stm:13.3

# 벤더 생략 시 프로젝트 MCU로 자동 추론
$ embtool toolchain install 14.2    # Kinetis 프로젝트 → nxp
```

#### `embtool toolchain list`
```bash
$ embtool toolchain list
Installed toolchains:
  * nxp-14.2.1   gcc 14.2.1   (450 MB)   [NXP MCUXpresso]
    nxp-13.2.1   gcc 13.2.1   (480 MB)   [NXP MCUXpresso]
    stm-13.3.1   gcc 13.3.1   (420 MB)   [STM32CubeIDE]

$ embtool toolchain list --available
Available toolchains (remote):
  Vendor  Version  GCC     Source                    Win   Linux
  ──────  ───────  ──────  ────────────────────────  ────  ─────
  nxp     14.2.1   14.2.1  MCUXpresso IDE 25.6.136  ✅    ✅
  nxp     13.2.1   13.2.1  MCUXpresso IDE            ✅    -
  stm     13.3.1   13.3.1  STM32CubeIDE 1.0.100     ✅    -
```

#### `embtool toolchain use <vendor:version>`
```bash
$ embtool toolchain use stm:13.3
🔄 Switched to STM ARM GCC 13.3.1
   arm-none-eabi-gcc 13.3.1
```

#### `embtool toolchain remove <vendor:version>`
```bash
$ embtool toolchain remove nxp:13.2
🗑️  Removed NXP ARM GCC 13.2.1
   Freed 480 MB
```

#### 저장 구조
```
~/.embtool/
├── config.toml                    # 전역 설정
├── toolchains/                    # 벤더-버전 형식
│   ├── nxp-14.2.1/
│   │   ├── arm-none-eabi/
│   │   ├── bin/
│   │   ├── lib/
│   │   ├── redlib/                # NXP 전용
│   │   └── features/              # NXP 전용
│   ├── nxp-13.2.1/
│   └── stm-13.3.1/
└── cache/                         # 다운로드 캐시
    ├── versions.json
    └── *.7z
```

#### 전역 설정 (~/.embtool/config.toml)
```toml
[toolchain]
default = "nxp-14.2.1"

[registry]
url = "https://pub-25d9755030a54c3280b7a9f68e9bf67c.r2.dev"
cache_ttl_hours = 24               # versions.json 캐시 유효기간

# 사내 NAS 미러 (선택, 우선순위 높음)
[mirror]
enabled = false
type = "local"                     # "local" | "http"
url = "\\\\nas\\share\\embtool\\toolchains"   # Windows UNC
# url = "/mnt/nas/embtool/toolchains"         # Linux mount
fallback = true                    # 미러 실패 시 R2로

[debug]
default_probe = "pemicro"

[ci]
auto_detect = true
```

#### 팀 전역 설정 공유
```
프로젝트별 .embtool/config.toml 로 오버라이드:
project/
├── .embtool/
│   └── config.toml      # 프로젝트별 설정 (Git 커밋)
└── embtool.toml          # 프로젝트 메타데이터 (Git 커밋)
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
vendor = "nxp"                   # "nxp" | "stm"
version = "14.2.1"               # 프로젝트 고정 버전

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
# Toolchain: NXP ARM GCC 14.2.1
# Run 'embtool setup' to regenerate

cmake_minimum_required(VERSION 3.20)

# embtool managed toolchain path
set(EMBTOOL_HOME "$ENV{HOME}/.embtool")
set(EMBTOOL_TOOLCHAIN_ID "nxp-14.2.1")
set(ARM_TOOLCHAIN_ROOT "${EMBTOOL_HOME}/toolchains/${EMBTOOL_TOOLCHAIN_ID}")
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
    ├── download.rs            # HTTP 다운로드 + SHA256 검증 + 진행바
    ├── archive.rs             # 7z 아카이브 해제
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
sha2 = "0.10"                                        # SHA256 검증
serde_json = "1"                                     # versions.json 파싱
```

---

## 8. 구현 로드맵

### Phase 0: 팀 인프라 (Week 1)
- [ ] `~/.embtool` 디렉토리 구조 설계
- [ ] `config.toml` 전역/프로젝트별 설정 관리
- [ ] `embtool setup` 명령 (embtool.toml 읽고 환경 구성)
- [ ] `--ci` 플래그 / CI 환경 자동 감지
- [ ] Windows + Linux 경로 처리
- [ ] GitHub Releases 바이너리 배포 (cross-compilation)

### Phase 1: 툴체인 관리 (Week 2-3)
- [ ] 원격 versions.json 레지스트리 파싱 (Cloudflare R2)
- [ ] 멀티벤더 지원 (NXP, STM)
- [ ] NAS 미러 우선 다운로드 + R2 fallback
- [ ] 다운로드 + SHA256 검증 + 진행바 (indicatif)
- [ ] 7z 아카이브 해제 및 설치
- [ ] `toolchain install/list/use/remove` 구현 (vendor:version 형식)
- [ ] 버전 전환 (config 기반)

### Phase 2: 프로젝트 생성 (Week 4-5)
- [ ] MCU 데이터베이스 구축 (NXP Kinetis)
- [ ] 프로젝트 템플릿 시스템 (handlebars)
- [ ] `embtool new --mcu` 구현
- [ ] CMakeLists.txt / arm-toolchain.cmake 자동 생성
- [ ] embtool.toml 생성
- [ ] NXP 스타트업/링커스크립트 번들
- [ ] `--type bootloader/application/library` 지원

### Phase 3: 빌드 & 플래시 (Week 6-7)
- [ ] CMake 래핑 (`embtool build`)
- [ ] 빌드 결과 요약 (Flash/RAM 사용량 파싱)
- [ ] PEMicro 연동 (`embtool flash`)
- [ ] 빌드 프로파일 (Debug/Release)

### Phase 4: 마이그레이션 (Week 8-9)
- [ ] 기존 CMakeLists.txt + arm-toolchain.cmake 파싱
- [ ] 자동 embtool.toml 생성
- [ ] a2750mcu 프로젝트 마이그레이션 검증
- [ ] 빌드 결과 바이너리 비교 (diff)

### Phase 5: 고급 기능 (향후)
- [ ] 컴포넌트/패키지 관리 (components/ → 독립 패키지)
- [ ] STM32 지원
- [ ] GitLab CI / Jenkins 파이프라인 템플릿 생성
- [ ] VS Code 확장
- [ ] `embtool doctor` (환경 진단)

---

## 9. 차별화 포인트

| 기존 도구 | embtool |
|-----------|---------|
| IDE 종속 (MCUXpresso, KDS) | IDE 독립, CLI 기반 |
| 툴체인이 프로젝트 내부 (187MB) | 시스템 레벨 관리 (nvm 방식) |
| 수동 CMake 작성 | 자동 생성 |
| 벤더별 별도 도구 | NXP + STM32 통합 |
| Windows 중심 | Windows 개발 + Linux CI/CD |
| 개인 환경 구성 | `embtool setup` 팀 환경 자동 통일 |
| 수동 서버 관리 | R2 자동 다운로드 + NAS 미러 fallback |
| 팀원 온보딩 반나절 | `embtool setup` 5분 |

---

## 10. 릴리스 전략

### 바이너리 배포
```
GitHub Releases:
├── embtool-v0.1.0-windows-x86_64.zip    # Windows 팀원용
├── embtool-v0.1.0-linux-x86_64.tar.gz   # CI/CD 서버용
└── embtool-v0.1.0-linux-aarch64.tar.gz  # ARM CI 서버용 (선택)

사내 배포:
├── NAS: \\nas\share\tools\embtool\      # 사내 미러
└── GitLab: 패키지 레지스트리 (선택)
```

### CI/CD 크로스 컴파일 (GitHub Actions)
```yaml
# .github/workflows/release.yml
strategy:
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: x86_64-pc-windows-msvc
        os: windows-latest
```

---

*기획서 v0.3 - 2026-03-08*
