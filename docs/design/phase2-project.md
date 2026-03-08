# Phase 2: 프로젝트 생성

> Week 4-5 · Unit 2-1 ~ 2-3

---

## Unit 2-1: MCU 데이터베이스 (`mcu/nxp.rs`, `core/mcu_db.rs`)

### 목적
MCU 별칭 → 메타데이터 조회, 프로젝트 생성 시 자동 설정

### 데이터 구조

```rust
// mcu/nxp.rs
pub struct McuInfo {
    pub id: &'static str,           // "k64" (CLI 별칭)
    pub part_number: &'static str,  // "MK64FN1M0VLL12"
    pub define: &'static str,       // "MK64F12" (컴파일 -D 매크로)
    pub core: &'static str,         // "cortex-m4"
    pub fpu: &'static str,          // "soft" | "hard" | "softfp"
    pub clock_mhz: u32,             // 120
    pub flash_kb: u32,              // 1024
    pub ram_kb: u32,                // 256
    pub family: &'static str,       // "kinetis"
    pub series: &'static str,       // "K64"
}
```

### 내장 MCU 데이터

```rust
pub const NXP_MCUS: &[McuInfo] = &[
    McuInfo {
        id: "k10d", part_number: "MK10DN512VLL10", define: "MK10D7",
        core: "cortex-m4", fpu: "soft", clock_mhz: 100,
        flash_kb: 512, ram_kb: 128, family: "kinetis", series: "K10",
    },
    McuInfo {
        id: "k12", part_number: "MK12DN512VLH5", define: "MK12D5",
        core: "cortex-m4", fpu: "soft", clock_mhz: 50,
        flash_kb: 512, ram_kb: 128, family: "kinetis", series: "K12",
    },
    McuInfo {
        id: "k22f", part_number: "MK22FX512VLL12", define: "MK22F12",
        core: "cortex-m4", fpu: "soft", clock_mhz: 120,
        flash_kb: 512, ram_kb: 128, family: "kinetis", series: "K22",
    },
    McuInfo {
        id: "k64", part_number: "MK64FN1M0VLL12", define: "MK64F12",
        core: "cortex-m4", fpu: "soft", clock_mhz: 120,
        flash_kb: 1024, ram_kb: 256, family: "kinetis", series: "K64",
    },
    McuInfo {
        id: "k66", part_number: "MK66FN2M0VMD18", define: "MK66F18",
        core: "cortex-m4", fpu: "hard", clock_mhz: 180,
        flash_kb: 2048, ram_kb: 256, family: "kinetis", series: "K66",
    },
];
```

### 함수 설계 (`core/mcu_db.rs`)

```rust
/// 별칭으로 MCU 조회 ("k64" → McuInfo)
pub fn lookup(id: &str) -> Option<&'static McuInfo>

/// 전체 MCU 목록
pub fn list_all() -> &'static [McuInfo]

/// 패밀리 필터 ("kinetis" → [k10d, k12, k22f, k64, k66])
pub fn list_by_family(family: &str) -> Vec<&'static McuInfo>

/// 지원 MCU 별칭 목록 (에러 메시지용)
pub fn supported_ids() -> Vec<&'static str>
```

### 의존성
- 없음 (순수 데이터)

### 테스트 전략
- 모든 MCU 별칭 조회 성공 확인
- 존재하지 않는 별칭 → None
- list_by_family 필터 동작 확인

---

## Unit 2-2: 프로젝트 템플릿 엔진 (`core/template.rs`)

### 목적
MCU 정보 + 프로젝트명으로 전체 프로젝트 파일 자동 생성

### 프로젝트 타입

```rust
pub enum ProjectType {
    Application,   // 기본: main.c + startup + linker (전체 Flash)
    Bootloader,    // main에서 boot 로직, linker (Flash 앞부분)
    Library,       // .a 라이브러리, main.c 없음
}
```

### 생성 파일 목록

| 파일 | 설명 | 템플릿 변수 |
|------|------|------------|
| `embtool.toml` | 프로젝트 설정 | name, mcu.*, toolchain_version |
| `CMakeLists.txt` | 빌드 정의 | name, mcu.define, mcu.core, flags |
| `arm-toolchain.cmake` | 툴체인 설정 | toolchain_version |
| `system/startup.c` | MCU 스타트업 | mcu.define |
| `system/linkerscript.ld` | 링커 스크립트 | mcu.flash_kb, mcu.ram_kb |
| `system/system_{mcu}.c` | 시스템 초기화 | mcu.define, mcu.clock_mhz |
| `src/main.c` | 메인 소스 | (없음) |
| `.gitignore` | Git 무시 목록 | (없음) |

### 함수 설계

```rust
/// 프로젝트 전체 생성
pub fn generate_project(
    name: &str,
    mcu: &McuInfo,
    project_type: ProjectType,
    toolchain_version: &str,
    output_dir: &Path,
) -> Result<Vec<PathBuf>>      // 생성된 파일 목록 반환

/// arm-toolchain.cmake만 (재)생성 (setup에서 사용)
pub fn generate_toolchain_cmake(
    toolchain_version: &str,
    output_dir: &Path,
) -> Result<PathBuf>
```

### 프로젝트 타입별 차이

| | Application | Bootloader | Library |
|---|---|---|---|
| main.c | `while(1) {}` 루프 | boot 로직 + 점프 | 없음 |
| linkerscript | 전체 Flash/RAM | Flash 앞부분만 (예: 32KB) | N/A |
| CMake 출력 | .elf → .bin | .elf → .bin | .a |
| startup.c | 전체 벡터 테이블 | 최소 벡터 테이블 | 없음 |

### 템플릿 방식
- 단순 `format!()` / string replace 사용 (handlebars 불필요할 수 있음)
- 템플릿은 Rust 소스에 `include_str!()` 또는 `const &str`로 내장

### 생성되는 CMakeLists.txt 핵심부

```cmake
cmake_minimum_required(VERSION 3.20)

set(CMAKE_TOOLCHAIN_FILE "${CMAKE_SOURCE_DIR}/arm-toolchain.cmake")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project({name} C ASM)
set(EXECUTABLE ${PROJECT_NAME}.elf)

# Target MCU flags
add_compile_options(
    -D{mcu_define}
    -mcpu={core}
    -mfloat-abi={fpu}
    -mthumb
    -ffunction-sections
    -fdata-sections
    -fno-common
    -fsigned-char
)

# Sources — 사용자가 여기에 추가
file(GLOB_RECURSE SOURCES "src/*.c" "system/*.c")
add_executable(${EXECUTABLE} ${SOURCES})

# Linker
set(LINKER_SCRIPT "${CMAKE_SOURCE_DIR}/system/linkerscript.ld")
target_link_options(${EXECUTABLE} PRIVATE
    -T${LINKER_SCRIPT}
    -mcpu={core} -mthumb
    -Wl,--gc-sections
    -specs=nano.specs -specs=nosys.specs
)
```

### 의존성
- Unit 2-1 (`mcu_db`)

### 테스트 전략
- 각 MCU × 각 프로젝트 타입 조합으로 생성 → 파일 존재 확인
- 생성된 CMakeLists.txt에 올바른 MCU define 포함 확인
- 생성된 embtool.toml 파싱 가능 확인 (round-trip)

---

## Unit 2-3: `embtool new` 명령 (`commands/new.rs`)

### CLI 정의

```rust
/// Create a new embedded project
New {
    /// Project name
    name: String,

    /// Target MCU alias (k10d, k22f, k64, k66)
    #[arg(long)]
    mcu: String,

    /// Project type
    #[arg(long, default_value = "application")]
    r#type: String,     // application | bootloader | library

    /// Toolchain version to use
    #[arg(long)]
    toolchain: Option<String>,
}
```

### 실행 플로우

```
1. mcu_db::lookup(mcu)
   → 없으면 에러 + 지원 목록 출력
   → "Unknown MCU 'xxx'. Supported: k10d, k12, k22f, k64, k66"

2. 출력 디렉토리 확인
   → ./{name} 이미 존재하면 에러

3. toolchain 버전 결정
   → --toolchain 옵션 있으면 사용
   → 없으면 config.toolchain.default 사용
   → 없으면 latest_version() 사용

4. template::generate_project() 호출

5. 결과 트리 출력
```

### 출력 예시

```
🆕 Creating project 'a2750lm_application'
   MCU: MK64FN1M0VLL12 (Cortex-M4, 120MHz, 1MB Flash, 256KB RAM)
   Toolchain: 13.3.rel1
   Type: application

📁 Generated:
   a2750lm_application/
   ├── embtool.toml
   ├── CMakeLists.txt
   ├── arm-toolchain.cmake
   ├── system/
   │   ├── startup.c
   │   ├── linkerscript.ld
   │   └── system_MK64F12.c
   └── src/
       └── main.c

✅ Project created! Next:
   cd a2750lm_application
   embtool setup
   embtool build
```

### 의존성
- Unit 2-1 (`mcu_db`)
- Unit 2-2 (`template`)
- Unit 0-2 (`config` — default toolchain)
