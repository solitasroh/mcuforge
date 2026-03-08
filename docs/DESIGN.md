# embtool 단위 기능 설계서

> **버전**: v1.0
> **작성일**: 2026-03-08
> **기반**: SPECIFICATION.md v0.2

---

## 설계 원칙

- **단일 책임**: 각 모듈은 하나의 역할만 수행
- **테스트 가능**: 모든 core 모듈은 유닛 테스트 가능하도록 설계
- **의존성 방향**: commands → core → utils (역방향 금지)
- **에러 처리**: `anyhow::Result` 통일, 사용자 친화적 에러 메시지

---

## Phase 0: 기반 인프라

### Unit 0-1: 경로 관리 (`utils/paths.rs`)

**목적**: `~/.embtool` 디렉토리 구조 생성 및 경로 해석

**구현 함수:**
```rust
pub fn embtool_home() -> PathBuf
// - Win: %USERPROFILE%/.embtool
// - Linux: $HOME/.embtool
// - 환경변수 EMBTOOL_HOME 오버라이드 지원

pub fn toolchains_dir() -> PathBuf   // ~/.embtool/toolchains/
pub fn cache_dir() -> PathBuf        // ~/.embtool/cache/
pub fn global_config() -> PathBuf    // ~/.embtool/config.toml
pub fn ensure_dirs() -> Result<()>   // 디렉토리 없으면 생성
```

**의존성**: `dirs` crate
**테스트**: 환경변수 mock으로 경로 검증

---

### Unit 0-2: 전역 설정 관리 (`core/config.rs`)

**목적**: `~/.embtool/config.toml` 읽기/쓰기/기본값

**데이터 구조:**
```rust
#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    pub toolchain: ToolchainConfig,
    pub debug: DebugConfig,
    pub ci: CiConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ToolchainConfig {
    pub default: Option<String>,           // "13.3.rel1"
    pub install_dir: Option<String>,       // 기본: ~/.embtool/toolchains
}

#[derive(Serialize, Deserialize)]
pub struct MirrorConfig {
    pub enabled: bool,                     // FTP 미러 사용 여부
    pub url: String,                       // "ftp://192.168.x.x/toolchains/"
    pub fallback: bool,                    // 미러 실패 시 ARM 공식 사용
}

#[derive(Serialize, Deserialize)]
pub struct DebugConfig {
    pub default_probe: String,             // "pemicro"
}

#[derive(Serialize, Deserialize)]
pub struct CiConfig {
    pub auto_detect: bool,                 // GITLAB_CI, JENKINS_URL 감지
}
```

**구현 함수:**
```rust
pub fn load() -> Result<GlobalConfig>       // 파일 읽기, 없으면 기본값
pub fn save(config: &GlobalConfig) -> Result<()>
pub fn is_ci() -> bool                      // CI 환경 감지
```

**의존성**: `serde`, `toml`, Unit 0-1

---

### Unit 0-3: 프로젝트 설정 관리 (`core/project.rs`)

**목적**: `embtool.toml` 파싱 및 검증

**데이터 구조:**
```rust
#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    pub target: TargetConfig,
    pub toolchain: ProjectToolchain,
    pub build: BuildConfig,
    pub debug: ProjectDebug,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct TargetConfig {
    pub mcu: String,          // "MK64FN1M0VLL12"
    pub core: String,         // "cortex-m4"
    pub fpu: String,          // "soft" | "hard" | "softfp"
    pub flash: String,        // "1M"
    pub ram: String,          // "256K"
}

#[derive(Serialize, Deserialize)]
pub struct ProjectToolchain {
    pub version: String,      // "13.3.rel1"
}

#[derive(Serialize, Deserialize)]
pub struct BuildConfig {
    pub c_standard: String,              // "c99"
    pub optimization: OptimizationConfig,
    pub linker_script: String,
    pub defines: DefinesConfig,
    pub flags: FlagsConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectDebug {
    pub probe: String,        // "pemicro"
    pub interface: String,    // "swd"
}
```

**구현 함수:**
```rust
pub fn find_project() -> Result<PathBuf>           // 현재~상위 디렉토리에서 embtool.toml 탐색
pub fn load(path: &Path) -> Result<ProjectConfig>  // 파싱 + 검증
pub fn save(path: &Path, config: &ProjectConfig) -> Result<()>
pub fn validate(config: &ProjectConfig) -> Result<()>  // MCU DB 대조 검증
```

**의존성**: `serde`, `toml`, Unit 0-1

---

### Unit 0-4: `embtool setup` 명령 (`commands/setup.rs`)

**목적**: embtool.toml 읽기 → 툴체인 설치 확인/설치 → cmake 생성

**플로우:**
```
1. find_project() → embtool.toml 경로 찾기
2. load() → 프로젝트 설정 파싱
3. 필요 툴체인 버전 확인
4. toolchains_dir()에 해당 버전 존재하는지 확인
5. 없으면 → toolchain install 호출
6. arm-toolchain.cmake 생성/재생성
7. 완료 메시지
```

**CLI 옵션:**
```rust
/// Set up project environment
Setup {
    #[arg(long)]
    ci: bool,        // CI 모드 (비대화형)
    #[arg(long)]
    force: bool,     // 강제 재설정
}
```

**의존성**: Unit 0-2, 0-3, Phase 1 (toolchain install)

---

## Phase 1: 툴체인 관리

### Unit 1-1: ARM 툴체인 URL 레지스트리 (`core/toolchain_registry.rs`)

**목적**: 버전 → 다운로드 URL 매핑

**ARM GNU Toolchain URL 패턴:**
```
# 최신 (13.x~)
https://developer.arm.com/-/media/Files/downloads/gnu/{version}/binrel/
  arm-gnu-toolchain-{version}-x86_64-arm-none-eabi.tar.xz      (Linux x86_64)
  arm-gnu-toolchain-{version}-mingw-w64-i686-arm-none-eabi.zip  (Windows)
  arm-gnu-toolchain-{version}-aarch64-arm-none-eabi.tar.xz      (Linux aarch64)

# 예시: 13.3.rel1
https://developer.arm.com/-/media/Files/downloads/gnu/13.3.rel1/binrel/
  arm-gnu-toolchain-13.3.rel1-x86_64-arm-none-eabi.tar.xz

# 구버전 (10.x~12.x)
https://developer.arm.com/-/media/Files/downloads/gnu/{version}/binrel/
  arm-gnu-toolchain-{version}-x86_64-arm-none-eabi.tar.xz
```

**데이터 구조:**
```rust
pub struct ToolchainRelease {
    pub version: String,          // "13.3.rel1"
    pub gcc_version: String,      // "13.3.1"
    pub url_linux_x64: String,
    pub url_linux_aarch64: String,
    pub url_windows: String,
    pub sha256_linux_x64: Option<String>,
}

// 내장 레지스트리 (코드에 하드코딩)
pub fn known_versions() -> Vec<ToolchainRelease>
pub fn resolve_url(version: &str) -> Result<String>  // OS 자동 감지
pub fn latest_version() -> &str
```

**내장 버전 목록:**
| 입력 | 정규화 | GCC | 상태 |
|------|--------|-----|------|
| `13.3` | `13.3.rel1` | 13.3.1 | latest |
| `13.2` | `13.2.rel1` | 13.2.1 | stable |
| `12.3` | `12.3.rel1` | 12.3.1 | stable |
| `12.2` | `12.2.rel1` | 12.2.1 | stable |
| `11.3` | `11.3.rel1` | 11.3.1 | legacy |
| `10.3` | `10.3-2021.10` | 10.3.1 | legacy |

**의존성**: 없음 (순수 데이터)

---

### Unit 1-2: 다운로드 엔진 (`utils/download.rs`)

**목적**: HTTP/FTP URL에서 파일 다운로드 + 진행바

**구현 함수:**
```rust
pub fn download_file(
    url: &str,
    dest: &Path,
    show_progress: bool,    // CI에서는 false
) -> Result<PathBuf>
```

**플로우:**
```
1. reqwest GET 요청
2. Content-Length 헤더로 총 크기 파악
3. indicatif ProgressBar 생성 (show_progress=true일 때)
4. 청크 단위 다운로드 → 파일 쓰기 + 진행바 업데이트
5. SHA256 체크섬 검증 (제공된 경우)
6. 캐시 디렉토리에 저장 (재다운로드 방지)
```

**캐시 전략:**
```
~/.embtool/cache/arm-gnu-toolchain-13.3.rel1-x86_64-arm-none-eabi.tar.xz
→ 이미 존재하면 다운로드 스킵
→ --force 시 재다운로드
```

**의존성**: `reqwest`, `indicatif`, Unit 0-1

---

### Unit 1-3: 아카이브 해제 (`utils/archive.rs`)

**목적**: tar.xz (Linux) / zip (Windows) 해제

**구현 함수:**
```rust
pub fn extract(
    archive_path: &Path,
    dest_dir: &Path,
    show_progress: bool,
) -> Result<PathBuf>        // 해제된 디렉토리 경로 반환
```

**플로우:**
```
1. 확장자 판단 (.tar.xz / .zip)
2. tar.xz: xz2로 디코딩 → tar로 해제
3. zip: zip crate로 해제
4. 해제된 루트 디렉토리 이름 감지
5. toolchains/{version}/ 으로 이동/리네임
```

**의존성**: `tar`, `xz2`, `zip` (새로 추가), Unit 0-1

---

### Unit 1-4: 툴체인 매니저 (`core/toolchain_manager.rs`)

**목적**: install/list/use/remove 핵심 로직

**구현 함수:**
```rust
pub fn install(version: &str, force: bool, ci: bool) -> Result<()>
// 1. resolve_url(version) → 다운로드 URL
// 2. 캐시 확인 → 없으면 download_file()
// 3. extract() → toolchains/{version}/
// 4. arm-none-eabi-gcc --version 으로 검증
// 5. 첫 설치면 default로 설정

pub fn list() -> Result<Vec<InstalledToolchain>>
// toolchains/ 디렉토리 스캔
// 각 버전의 gcc --version 실행
// active 버전 표시

pub fn use_version(version: &str) -> Result<()>
// config.toml의 default 값 변경

pub fn remove(version: &str) -> Result<()>
// 디렉토리 삭제
// active였으면 다른 버전으로 전환

pub fn get_active() -> Result<String>
// config.toml에서 default 읽기

pub fn get_toolchain_path(version: &str) -> Result<PathBuf>
// ~/.embtool/toolchains/{version}/bin/
```

**데이터 구조:**
```rust
pub struct InstalledToolchain {
    pub version: String,       // "13.3.rel1"
    pub gcc_version: String,   // "13.3.1 20240614"
    pub path: PathBuf,
    pub is_active: bool,
    pub size_mb: u64,
}
```

**의존성**: Unit 0-1, 0-2, 1-1, 1-2, 1-3

---

### Unit 1-5: `embtool toolchain` 명령들 (`commands/toolchain.rs`)

**목적**: CLI → core 연결

**구현**: 각 서브커맨드에서 `toolchain_manager`의 해당 함수 호출 + 출력 포맷팅

**의존성**: Unit 1-4, `colored`

---

## Phase 2: 프로젝트 생성

### Unit 2-1: MCU 데이터베이스 (`mcu/nxp.rs`, `core/mcu_db.rs`)

**목적**: MCU 별칭 → 메타데이터 조회

**데이터 구조:**
```rust
pub struct McuInfo {
    pub id: &'static str,           // "k64"
    pub part_number: &'static str,  // "MK64FN1M0VLL12"
    pub define: &'static str,       // "MK64F12"
    pub core: &'static str,         // "cortex-m4"
    pub fpu: &'static str,          // "soft"
    pub clock_mhz: u32,             // 120
    pub flash_kb: u32,              // 1024
    pub ram_kb: u32,                // 256
    pub family: &'static str,       // "kinetis"
    pub series: &'static str,       // "K64"
}
```

**내장 MCU 목록:**
```rust
pub const NXP_MCUS: &[McuInfo] = &[
    McuInfo { id: "k10d",  part_number: "MK10DN512VLL10", define: "MK10D7",  core: "cortex-m4", fpu: "soft", clock_mhz: 100, flash_kb: 512,  ram_kb: 128, family: "kinetis", series: "K10" },
    McuInfo { id: "k12",   part_number: "MK12DN512VLH5",  define: "MK12D5",  core: "cortex-m4", fpu: "soft", clock_mhz: 50,  flash_kb: 512,  ram_kb: 128, family: "kinetis", series: "K12" },
    McuInfo { id: "k22f",  part_number: "MK22FX512VLL12", define: "MK22F12", core: "cortex-m4", fpu: "soft", clock_mhz: 120, flash_kb: 512,  ram_kb: 128, family: "kinetis", series: "K22" },
    McuInfo { id: "k64",   part_number: "MK64FN1M0VLL12", define: "MK64F12", core: "cortex-m4", fpu: "soft", clock_mhz: 120, flash_kb: 1024, ram_kb: 256, family: "kinetis", series: "K64" },
    McuInfo { id: "k66",   part_number: "MK66FN2M0VMD18", define: "MK66F18", core: "cortex-m4", fpu: "hard", clock_mhz: 180, flash_kb: 2048, ram_kb: 256, family: "kinetis", series: "K66" },
];
```

**구현 함수:**
```rust
pub fn lookup(id: &str) -> Option<&McuInfo>     // "k64" → McuInfo
pub fn list_all() -> &[McuInfo]                 // 전체 목록
pub fn list_by_family(family: &str) -> Vec<&McuInfo>  // "kinetis" 필터
```

**의존성**: 없음

---

### Unit 2-2: 프로젝트 템플릿 엔진 (`core/template.rs`)

**목적**: MCU 정보 기반으로 프로젝트 파일 생성

**생성 파일 목록:**
| 파일 | 템플릿 소스 | 변수 |
|------|-----------|------|
| `embtool.toml` | 내장 문자열 | name, mcu.*, toolchain |
| `CMakeLists.txt` | 내장 문자열 | name, mcu.define, mcu.core, flags |
| `arm-toolchain.cmake` | 내장 문자열 | toolchain_version |
| `system/startup.c` | 내장 (MCU별) | mcu.define |
| `system/linkerscript.ld` | 내장 (MCU별) | mcu.flash_kb, mcu.ram_kb |
| `system/system_{mcu}.c` | 내장 (MCU별) | mcu.define, mcu.clock_mhz |
| `src/main.c` | 내장 문자열 | (없음) |
| `.gitignore` | 내장 문자열 | (없음) |

**구현 함수:**
```rust
pub fn generate_project(
    name: &str,
    mcu: &McuInfo,
    project_type: ProjectType,  // Application | Bootloader | Library
    toolchain_version: &str,
    output_dir: &Path,
) -> Result<()>
```

**프로젝트 타입별 차이:**
| | Application | Bootloader | Library |
|---|---|---|---|
| main.c | `int main(void) { while(1) {} }` | `void boot(void) {...}` | (없음) |
| linker | 전체 Flash/RAM | Flash 앞부분만 | N/A |
| output | .elf, .bin | .elf, .bin | .a |

**의존성**: Unit 2-1, `handlebars` (또는 간단한 string replace)

---

### Unit 2-3: `embtool new` 명령 (`commands/new.rs`)

**목적**: CLI → MCU 조회 → 템플릿 생성

**플로우:**
```
1. --mcu 인자로 mcu_db::lookup()
2. 없으면 에러 + 지원 MCU 목록 출력
3. 출력 디렉토리 생성
4. generate_project() 호출
5. 결과 트리 출력
```

**의존성**: Unit 2-1, 2-2

---

## Phase 3: 빌드 & 플래시

### Unit 3-1: CMake 래퍼 (`core/builder.rs`)

**목적**: CMake configure + build 실행, 결과 파싱

**구현 함수:**
```rust
pub fn build(
    project_dir: &Path,
    profile: BuildProfile,  // Debug | Release
    verbose: bool,
    clean: bool,
) -> Result<BuildResult>

pub struct BuildResult {
    pub success: bool,
    pub elf_path: PathBuf,
    pub flash_used: u32,
    pub flash_total: u32,
    pub ram_used: u32,
    pub ram_total: u32,
    pub build_time_secs: f64,
}
```

**플로우:**
```
1. find_project() → embtool.toml 읽기
2. toolchain 경로 확인 (없으면 embtool setup 안내)
3. clean이면 build/ 삭제
4. cmake -B build -DCMAKE_BUILD_TYPE={profile}
         -DCMAKE_TOOLCHAIN_FILE=arm-toolchain.cmake
5. cmake --build build
6. arm-none-eabi-size build/{name}.elf → Flash/RAM 파싱
7. BuildResult 반환
```

**size 출력 파싱:**
```
   text    data     bss     dec     hex filename
  42768    2544    9744   55056    d710 build/a2750lm_application.elf

Flash used = text + data
RAM used = data + bss
```

**의존성**: Unit 0-3, 1-4, `std::process::Command`

---

### Unit 3-2: 바이너리 변환 (`core/objcopy.rs`)

**목적**: .elf → .bin / .hex 변환

**구현 함수:**
```rust
pub fn elf_to_bin(elf_path: &Path) -> Result<PathBuf>
pub fn elf_to_hex(elf_path: &Path) -> Result<PathBuf>
// arm-none-eabi-objcopy -O binary input.elf output.bin
// arm-none-eabi-objcopy -O ihex input.elf output.hex
```

**의존성**: Unit 1-4 (toolchain path)

---

### Unit 3-3: PEMicro 플래시 (`core/flasher.rs`)

**목적**: PEMicro CLI로 .elf 플래시

**구현 함수:**
```rust
pub fn flash(
    elf_path: &Path,
    probe: &str,        // "pemicro"
    interface: &str,     // "swd"
) -> Result<()>
```

**의존성**: PEMicro CLI (`pegdbserver_console` 또는 동등), `std::process::Command`

> ⚠️ PEMicro CLI 도구 조사 필요 — Phase 3 시작 시 상세 설계

---

### Unit 3-4: `embtool build` / `embtool flash` 명령 (`commands/build.rs`, `commands/flash.rs`)

**목적**: CLI → core 연결 + 출력 포맷팅

**의존성**: Unit 3-1, 3-2, 3-3

---

## Phase 4: 마이그레이션

### Unit 4-1: CMake 파서 (`core/migrate_parser.rs`)

**목적**: 기존 CMakeLists.txt + arm-toolchain.cmake 분석

**추출 대상:**
```
CMakeLists.txt에서:
- project(name)         → project.name
- -D{MCU_DEFINE}        → target.mcu (MCU DB 역조회)
- -mcpu=cortex-m4       → target.core
- -mfloat-abi=soft      → target.fpu
- CMAKE_C_STANDARD      → build.c_standard
- -O0 / -O1             → build.optimization
- LINKER_FILE           → build.linker_script
- add_compile_options    → build.flags

arm-toolchain.cmake에서:
- ARM_TOOLCHAIN_PATH    → 현재 사용 중인 툴체인 감지
```

**구현 함수:**
```rust
pub fn analyze_cmake(dir: &Path) -> Result<MigrationPlan>

pub struct MigrationPlan {
    pub detected_mcu: Option<String>,
    pub detected_core: Option<String>,
    pub detected_fpu: Option<String>,
    pub detected_toolchain: Option<String>,
    pub detected_c_standard: Option<String>,
    pub detected_flags: Vec<String>,
    pub detected_defines: Vec<String>,
    pub linker_script: Option<String>,
    pub confidence: f32,           // 0.0 ~ 1.0
    pub warnings: Vec<String>,
}
```

**의존성**: regex, Unit 2-1

---

### Unit 4-2: `embtool migrate` 명령 (`commands/migrate.rs`)

**목적**: 분석 결과 표시 → 사용자 확인 → embtool.toml 생성

**플로우:**
```
1. analyze_cmake() 실행
2. 분석 결과 출력 (감지된 MCU, 코어, 플래그 등)
3. confidence 낮으면 경고
4. CI 모드 아니면 사용자 확인 (Y/n)
5. embtool.toml 생성
6. arm-toolchain.cmake 백업 (.bak) 후 재생성
7. 빌드 테스트 제안
```

**의존성**: Unit 4-1, 0-3, 2-2, `dialoguer`

---

## 모듈 의존성 그래프

```
commands/
  setup.rs      → core/project, core/toolchain_manager, core/template
  toolchain.rs  → core/toolchain_manager
  new.rs        → core/mcu_db, core/template
  build.rs      → core/builder
  flash.rs      → core/flasher
  migrate.rs    → core/migrate_parser, core/project, core/template

core/
  config.rs            → utils/paths
  project.rs           → utils/paths
  toolchain_manager.rs → config, toolchain_registry, utils/download, utils/archive
  toolchain_registry.rs→ (없음)
  mcu_db.rs            → mcu/nxp
  template.rs          → mcu_db
  builder.rs           → project, toolchain_manager
  flasher.rs           → project, toolchain_manager
  migrate_parser.rs    → mcu_db

mcu/
  nxp.rs  → (없음)
  stm32.rs → (없음, 향후)

utils/
  paths.rs    → (없음)
  download.rs → paths
  archive.rs  → paths
```

---

## 구현 순서 (의존성 기반)

```
Week 1:
  ① utils/paths.rs          (의존성 없음)
  ② core/config.rs          (← paths)
  ③ core/project.rs         (← paths)
  ④ mcu/nxp.rs              (의존성 없음)
  ⑤ core/mcu_db.rs          (← nxp)
  ⑥ commands/setup.rs       (← config, project) [스켈레톤]

Week 2:
  ⑦ core/toolchain_registry.rs  (의존성 없음)
  ⑧ utils/download.rs           (← paths)
  ⑨ utils/archive.rs            (← paths)
  ⑩ core/toolchain_manager.rs   (← registry, download, archive, config)

Week 3:
  ⑪ commands/toolchain.rs       (← toolchain_manager)
  ⑫ commands/setup.rs 완성      (← toolchain_manager 연동)
  → Phase 0+1 릴리스: v0.1.0

Week 4:
  ⑬ core/template.rs            (← mcu_db)
  ⑭ commands/new.rs             (← mcu_db, template)

Week 5:
  ⑮ NXP startup/linker 번들 작성
  → Phase 2 릴리스: v0.2.0

Week 6-7:
  ⑯ core/builder.rs             (← project, toolchain_manager)
  ⑰ core/objcopy.rs             (← toolchain_manager)
  ⑱ core/flasher.rs             (← project)
  ⑲ commands/build.rs, flash.rs
  → Phase 3 릴리스: v0.3.0

Week 8-9:
  ⑳ core/migrate_parser.rs      (← mcu_db)
  ㉑ commands/migrate.rs         (← migrate_parser, project, template)
  → Phase 4 릴리스: v0.4.0
```

---

## 파일 구조 (최종)

```
src/
├── main.rs
├── commands/
│   ├── mod.rs
│   ├── setup.rs
│   ├── toolchain.rs
│   ├── new.rs
│   ├── build.rs
│   ├── flash.rs
│   ├── config.rs
│   └── migrate.rs
├── core/
│   ├── mod.rs
│   ├── config.rs
│   ├── project.rs
│   ├── toolchain_manager.rs
│   ├── toolchain_registry.rs
│   ├── mcu_db.rs
│   ├── template.rs
│   ├── builder.rs
│   ├── objcopy.rs
│   ├── flasher.rs
│   └── migrate_parser.rs
├── mcu/
│   ├── mod.rs
│   ├── nxp.rs
│   └── stm32.rs
└── utils/
    ├── mod.rs
    ├── paths.rs
    ├── download.rs
    └── archive.rs
```

---

*설계서 v1.0 — 2026-03-08*
