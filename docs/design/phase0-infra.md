# Phase 0: 기반 인프라

> Week 1 · Unit 0-1 ~ 0-4

---

## Unit 0-1: 경로 관리 (`utils/paths.rs`)

### 목적
`~/.embtool` 디렉토리 구조 생성 및 OS별 경로 해석

### 함수 설계

```rust
/// embtool 홈 디렉토리 반환
/// - Win: %USERPROFILE%/.embtool
/// - Linux: $HOME/.embtool
/// - 환경변수 EMBTOOL_HOME 오버라이드 지원
pub fn embtool_home() -> PathBuf

/// ~/.embtool/toolchains/
pub fn toolchains_dir() -> PathBuf

/// ~/.embtool/cache/
pub fn cache_dir() -> PathBuf

/// ~/.embtool/config.toml
pub fn global_config_path() -> PathBuf

/// 디렉토리가 없으면 생성 (toolchains, cache 포함)
pub fn ensure_dirs() -> Result<()>
```

### 디렉토리 구조

```
~/.embtool/
├── config.toml           # 전역 설정
├── toolchains/           # 설치된 ARM GCC (벤더-버전 형식)
│   ├── nxp-14.2.1/
│   ├── nxp-13.2.1/
│   └── stm-13.3.1/
└── cache/                # 다운로드 캐시
    ├── versions.json
    └── *.7z
```

### 의존성
- `dirs` crate (홈 디렉토리 감지)

### 테스트 전략
- `EMBTOOL_HOME` 환경변수 설정 후 tempdir로 테스트
- 각 경로 함수가 올바른 하위 경로 반환하는지 검증
- `ensure_dirs()` 후 디렉토리 존재 확인

---

## Unit 0-2: 전역 설정 관리 (`core/config.rs`)

### 목적
`~/.embtool/config.toml` 읽기/쓰기/기본값 관리

### 데이터 구조

```rust
#[derive(Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub toolchain: ToolchainConfig,
    #[serde(default)]
    pub registry: RegistryConfig,
    #[serde(default)]
    pub mirror: MirrorConfig,
    #[serde(default)]
    pub debug: DebugConfig,
    #[serde(default)]
    pub ci: CiConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ToolchainConfig {
    pub default: Option<String>,       // "nxp-14.2.1"
    pub install_dir: Option<String>,   // 오버라이드 시에만 사용
}

#[derive(Serialize, Deserialize)]
pub struct RegistryConfig {
    pub url: String,                   // "https://pub-25d9755030a54c3280b7a9f68e9bf67c.r2.dev"
    pub cache_ttl_hours: u32,          // 24 (versions.json 캐시 유효기간)
}

#[derive(Serialize, Deserialize)]
pub struct MirrorConfig {
    pub enabled: bool,                 // false (기본)
    pub url: String,                   // "" (NAS 경로 또는 HTTP URL)
    pub mirror_type: String,           // "local" | "http"
    pub fallback: bool,               // true (미러 실패 시 R2로)
}

#[derive(Serialize, Deserialize)]
pub struct DebugConfig {
    pub default_probe: String,         // "pemicro"
}

#[derive(Serialize, Deserialize)]
pub struct CiConfig {
    pub auto_detect: bool,             // true
}
```

### 함수 설계

```rust
/// config.toml 로드 (없으면 기본값 반환)
pub fn load() -> Result<GlobalConfig>

/// config.toml 저장
pub fn save(config: &GlobalConfig) -> Result<()>

/// CI 환경 여부 감지
/// GITLAB_CI, JENKINS_URL, CI, GITHUB_ACTIONS 환경변수 확인
pub fn is_ci() -> bool
```

### 의존성
- Unit 0-1 (`paths::global_config_path()`)
- `serde`, `toml`

### 테스트 전략
- 빈 파일 → 기본값 로드 확인
- 저장 후 재로드 → 값 일치 확인
- CI 환경변수 mock 테스트

---

## Unit 0-3: 프로젝트 설정 관리 (`core/project.rs`)

### 목적
`embtool.toml` 파싱, 검증, 현재 디렉토리에서 탐색

### 데이터 구조

```rust
#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    pub target: TargetConfig,
    pub toolchain: ProjectToolchain,
    pub build: BuildConfig,
    #[serde(default)]
    pub debug: ProjectDebug,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,            // "a2750lm_application"
    pub version: String,         // "1.0.0"
}

#[derive(Serialize, Deserialize)]
pub struct TargetConfig {
    pub mcu: String,             // "MK64FN1M0VLL12"
    pub core: String,            // "cortex-m4"
    pub fpu: String,             // "soft"
    pub flash: String,           // "1M"
    pub ram: String,             // "256K"
}

#[derive(Serialize, Deserialize)]
pub struct ProjectToolchain {
    pub version: String,         // "13.3.rel1"
}

#[derive(Serialize, Deserialize)]
pub struct BuildConfig {
    pub c_standard: String,                   // "c99"
    #[serde(default)]
    pub optimization: OptimizationConfig,
    pub linker_script: String,
    #[serde(default)]
    pub defines: DefinesConfig,
    #[serde(default)]
    pub flags: FlagsConfig,
}

#[derive(Serialize, Deserialize, Default)]
pub struct OptimizationConfig {
    pub debug: String,           // "O0"
    pub release: String,         // "O1"
}

#[derive(Serialize, Deserialize, Default)]
pub struct DefinesConfig {
    pub target: Vec<String>,     // ["MK64F12"]
    pub custom: Vec<String>,     // []
}

#[derive(Serialize, Deserialize, Default)]
pub struct FlagsConfig {
    pub common: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectDebug {
    pub probe: String,           // "pemicro"
    pub interface: String,       // "swd"
}
```

### 함수 설계

```rust
/// 현재 디렉토리 ~ 루트까지 embtool.toml 탐색
pub fn find_project() -> Result<PathBuf>

/// embtool.toml 파싱
pub fn load(path: &Path) -> Result<ProjectConfig>

/// embtool.toml 저장
pub fn save(path: &Path, config: &ProjectConfig) -> Result<()>

/// MCU DB 대조 검증 (존재하는 MCU인지, 값 일관성)
pub fn validate(config: &ProjectConfig) -> Result<()>
```

### 탐색 로직
```
현재 디렉토리에서 embtool.toml 찾기
→ 없으면 상위 디렉토리로 이동
→ 루트까지 반복
→ 못 찾으면 에러: "embtool.toml not found. Run 'embtool new' first."
```

### 의존성
- Unit 0-1 (`paths`)
- `serde`, `toml`

### 테스트 전략
- 정상 toml 파싱 → 모든 필드 확인
- 필수 필드 누락 → 에러 확인
- 상위 디렉토리 탐색 동작 확인

---

## Unit 0-4: `embtool setup` 명령 (`commands/setup.rs`)

### 목적
프로젝트 디렉토리에서 embtool.toml을 읽고 필요한 환경을 자동 구성

### CLI 정의

```rust
/// Set up project environment (install toolchain, generate cmake)
Setup {
    /// CI mode — disable interactive prompts
    #[arg(long)]
    ci: bool,

    /// Force re-setup even if already configured
    #[arg(long)]
    force: bool,

    /// Override FTP mirror URL
    #[arg(long)]
    mirror: Option<String>,
}
```

### 실행 플로우

```
1. find_project()
   → embtool.toml 경로 찾기
   → 없으면 에러 + 안내 메시지

2. load()
   → 프로젝트 설정 파싱
   → 프로젝트명, MCU, 툴체인 버전 출력

3. 툴체인 확인
   → toolchains_dir()/{version} 존재하는지
   → 있으면 "Already installed" 스킵
   → 없으면 toolchain_manager::install() 호출

4. arm-toolchain.cmake 생성
   → 프로젝트 디렉토리에 생성/덮어쓰기
   → OS 자동 감지 cmake 코드

5. 완료 메시지
   → "Setup complete! Run 'embtool build'"
```

### 출력 예시

```
🔍 Reading embtool.toml...
   Project: a2750lm_application
   MCU: MK64FN1M0VLL12 (Cortex-M4)
   Toolchain: 13.3.rel1

📦 Toolchain 13.3.rel1 not found.
   Downloading from ARM official...
   [████████████████████████████████] 100% (245 MB)

🔧 Generating arm-toolchain.cmake...
✅ Setup complete! Run 'embtool build' to build.
```

### 의존성
- Unit 0-2 (`config`)
- Unit 0-3 (`project`)
- Unit 1-4 (`toolchain_manager`) — 툴체인 설치 부분
- `colored`

### 참고
- `--ci` 모드에서는 진행바 비활성화, 프롬프트 없음
- `--force` 시 기존 cmake 파일 무조건 덮어쓰기
- Phase 1 완료 후에 완전히 동작 가능 (의존성)
