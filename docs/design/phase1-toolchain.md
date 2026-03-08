# Phase 1: 툴체인 관리

> Week 2-3 · Unit 1-1 ~ 1-5

---

## Unit 1-1: 툴체인 레지스트리 (`core/toolchain_registry.rs`)

### 목적
원격 `versions.json`에서 툴체인 메타데이터 조회, 벤더별 버전 관리

### 레지스트리 URL

```
기본 URL (Cloudflare R2):
https://pub-25d9755030a54c3280b7a9f68e9bf67c.r2.dev/

메타데이터 리포 (참조):
https://github.com/solitasroh/embtool-toolchains
```

### versions.json 스키마

```json
{
  "schema_version": 1,
  "latest": {
    "nxp": "14.2.1",
    "stm": "13.3.1"
  },
  "toolchains": [
    {
      "version": "14.2.1",
      "vendor": "nxp",
      "gcc": "14.2.1",
      "source": "MCUXpresso IDE 25.6.136",
      "date": "2025-06",
      "includes": ["redlib", "nxp-features", "newlib-nano"],
      "assets": {
        "linux-x64": {
          "file": "nxp-14.2.1-linux-x64.7z",
          "size": 143668877,
          "sha256": "2980c54a..."
        },
        "win-x64": {
          "file": "nxp-14.2.1-win-x64.7z",
          "size": 122777837,
          "sha256": "885a2840..."
        }
      }
    }
  ]
}
```

### 데이터 구조

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionsManifest {
    pub schema_version: u32,
    pub latest: HashMap<String, String>,   // {"nxp": "14.2.1", "stm": "13.3.1"}
    pub toolchains: Vec<ToolchainEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainEntry {
    pub version: String,          // "14.2.1"
    pub vendor: String,           // "nxp" | "stm"
    pub gcc: String,              // "14.2.1"
    pub source: String,           // "MCUXpresso IDE 25.6.136"
    pub date: String,             // "2025-06"
    pub includes: Vec<String>,    // ["redlib", "nxp-features", "newlib-nano"]
    pub assets: HashMap<String, Option<AssetInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub file: String,             // "nxp-14.2.1-linux-x64.7z"
    pub size: u64,                // bytes
    pub sha256: String,
}

/// 설치된 툴체인 표현
pub struct InstalledToolchain {
    pub version: String,          // "14.2.1"
    pub vendor: String,           // "nxp"
    pub gcc_version: String,      // "14.2.1"
    pub path: PathBuf,            // ~/.embtool/toolchains/nxp-14.2.1/
    pub is_active: bool,
    pub size_mb: u64,
}
```

### 함수 설계

```rust
/// 레지스트리에서 versions.json 가져오기
/// 우선순위: mirror (NAS) → R2 (기본)
pub fn fetch_manifest(config: &GlobalConfig) -> Result<VersionsManifest>

/// 벤더+버전으로 툴체인 엔트리 찾기
pub fn find_toolchain(manifest: &VersionsManifest, vendor: &str, version: &str) -> Result<&ToolchainEntry>

/// 현재 OS에 맞는 asset 조회
pub fn resolve_asset(entry: &ToolchainEntry) -> Result<&AssetInfo>

/// 특정 벤더의 최신 버전 반환
pub fn latest_version(manifest: &VersionsManifest, vendor: &str) -> Result<&str>

/// 사용 가능한 모든 버전 나열
pub fn available_versions(manifest: &VersionsManifest) -> Vec<(&str, &str, &str)>
// → [("nxp", "14.2.1", "14.2.1"), ("nxp", "13.2.1", "13.2.1"), ("stm", "13.3.1", "13.3.1")]
```

### 다운로드 URL 조합

```rust
fn download_url(config: &GlobalConfig, asset: &AssetInfo) -> String {
    let base = if config.mirror.enabled {
        &config.mirror.url
    } else {
        &config.registry.url  // https://pub-...r2.dev
    };
    format!("{}/{}", base.trim_end_matches('/'), asset.file)
}
```

### 캐시 정책
- `versions.json`은 `~/.embtool/cache/versions.json`에 캐시
- 유효기간: 24시간 (기본), `--refresh` 플래그로 강제 갱신
- 오프라인 시 캐시된 버전 사용

### 의존성
- Unit 0-1 (`paths::cache_dir()`)
- Unit 0-2 (`config`)
- `reqwest` (rustls-tls), `serde_json`

### 테스트 전략
- 샘플 versions.json 파싱 검증
- 벤더+버전 검색 로직 테스트
- 존재하지 않는 OS asset → 에러 확인

---

## Unit 1-2: 다운로드 엔진 (`utils/download.rs`)

### 목적
HTTP URL에서 파일 다운로드, SHA256 검증, 진행바 표시, 캐시 관리

### 함수 설계

```rust
/// URL에서 파일 다운로드
/// - SHA256 해시 검증 포함
/// - 캐시 히트 시 다운로드 스킵 (해시 일치 확인)
/// - 반환: 다운로드된 파일 경로
pub fn download_file(
    url: &str,
    dest_dir: &Path,           // ~/.embtool/cache/
    filename: &str,
    expected_sha256: &str,
    show_progress: bool,
) -> Result<PathBuf>

/// SHA256 해시 검증
pub fn verify_sha256(path: &Path, expected: &str) -> Result<bool>

/// 캐시에 파일 존재하는지 확인 (해시 포함)
pub fn is_cached(filename: &str, expected_sha256: &str) -> bool

/// 캐시 디렉토리 크기 조회 (MB)
pub fn cache_size_mb() -> Result<u64>

/// 캐시 정리
pub fn clear_cache() -> Result<()>
```

### 다운로드 플로우

```
1. dest_dir/filename 존재 && SHA256 일치? → 스킵 ("Using cached ...")
2. reqwest::blocking::get(url)
3. Content-Length 헤더 → 총 크기
4. indicatif::ProgressBar 생성
   - 스타일: [████████████████████████████████] 100% (145/145 MB)
   - CI 모드: 진행바 비활성화 (println으로 대체)
5. 8KB 청크로 읽으며 파일 쓰기 + 진행바 + SHA256 해시 누적
6. 다운로드 완료 → SHA256 검증
   - 불일치 → 파일 삭제 + 에러
   - 일치 → 경로 반환
```

### 다운로드 소스 우선순위

```
1. mirror.enabled == true?
   → mirror URL에서 다운로드 시도
   → 실패 && mirror.fallback == true?
     → R2에서 다운로드
   → 실패 && fallback == false?
     → 에러
2. mirror.enabled == false
   → R2에서 바로 다운로드
```

### 에러 처리
- 네트워크 오류 → 재시도 1회
- 404 → "Toolchain not found on server"
- SHA256 불일치 → "Download corrupted, please retry"
- 디스크 공간 부족 → 사전 체크 (Content-Length 비교)

### 의존성
- Unit 0-1 (`paths::cache_dir()`)
- `reqwest` (blocking, rustls-tls)
- `indicatif`
- `sha2` (SHA256 검증)

### 테스트 전략
- 소형 테스트 파일 다운로드 (httpbin 등)
- SHA256 검증 성공/실패 테스트
- 캐시 히트 시 스킵 확인
- 잘못된 URL → 에러 확인

---

## Unit 1-3: 아카이브 해제 (`utils/archive.rs`)

### 목적
다운로드된 7z 아카이브를 툴체인 디렉토리에 해제

### 함수 설계

```rust
/// 아카이브 해제
/// - .7z → 7z 명령어 호출 (외부 의존)
/// - 반환: 해제된 루트 디렉토리 경로
pub fn extract(
    archive_path: &Path,
    dest_dir: &Path,          // ~/.embtool/toolchains/
    toolchain_name: &str,     // "nxp-14.2.1"
    show_progress: bool,
) -> Result<PathBuf>

/// 7z 명령어 존재 여부 확인
pub fn check_7z() -> Result<()>
```

### 해제 플로우

```
1. 7z 실행 가능한지 확인 (which 7z / where 7z)
   → 없으면: "7z not found. Install p7zip-full (Linux) or 7-Zip (Windows)"
2. 임시 디렉토리에 해제: 7z x archive.7z -o{tmpdir}
3. 해제 결과를 {dest_dir}/{toolchain_name}/ 으로 이동
4. arm-none-eabi-gcc 바이너리 존재 확인 (검증)
5. 임시 디렉토리 정리
```

### 디렉토리 매핑

```
아카이브 내부 (루트):             → 설치 경로:
├── arm-none-eabi/                → ~/.embtool/toolchains/nxp-14.2.1/
├── bin/                             ├── arm-none-eabi/
├── lib/                             ├── bin/
├── redlib/ (NXP only)               ├── lib/
├── features/ (NXP only)             ├── redlib/
└── ...                              └── ...
```

### 의존성
- Unit 0-1 (`paths::toolchains_dir()`)
- 외부: `7z` (p7zip-full / 7-Zip)

### 7z 의존성 처리
- `embtool setup` 시 7z 존재 여부 사전 확인
- 설치 안내 메시지 제공
- 향후: Rust 7z 라이브러리 (`sevenz-rust`)로 내장 가능

### 테스트 전략
- 소형 7z 생성 후 해제 검증
- 해제 후 기대 파일 존재 확인
- 7z 미설치 시 에러 메시지 확인

---

## Unit 1-4: 툴체인 매니저 (`core/toolchain_manager.rs`)

### 목적
install/list/use/remove 핵심 비즈니스 로직

### 함수 설계

```rust
/// 툴체인 설치
pub fn install(vendor: &str, version: &str, force: bool, ci: bool) -> Result<()>
// 1. fetch_manifest() → versions.json 가져오기
// 2. find_toolchain(vendor, version) → 엔트리 찾기
// 3. resolve_asset() → 현재 OS에 맞는 파일
// 4. 이미 설치됨? → force 아니면 스킵
// 5. download_file() (SHA256 검증 포함)
// 6. archive::extract()
// 7. verify_installation() — gcc --version 실행
// 8. 첫 설치면 config에 default 설정

/// 설치된 툴체인 목록
pub fn list() -> Result<Vec<InstalledToolchain>>
// 1. toolchains_dir() 스캔 (nxp-14.2.1/, stm-13.3.1/ 등)
// 2. 각 디렉토리에서 bin/arm-none-eabi-gcc --version 실행
// 3. config.default와 비교하여 is_active 설정

/// 기본 툴체인 변경
pub fn use_version(vendor: &str, version: &str) -> Result<()>
// 1. 설치 여부 확인
// 2. config.toml의 toolchain.default 업데이트

/// 툴체인 삭제
pub fn remove(vendor: &str, version: &str) -> Result<()>
// 1. active 버전이면 경고
// 2. 디렉토리 삭제
// 3. active였으면 다른 버전으로 전환 (또는 None)

/// 특정 버전의 bin/ 경로 반환
pub fn get_toolchain_bin_path(vendor: &str, version: &str) -> Result<PathBuf>
// ~/.embtool/toolchains/{vendor}-{version}/bin/

/// 현재 active 버전 반환
pub fn get_active() -> Result<Option<(String, String)>>
// → Some(("nxp", "14.2.1"))

/// 설치 검증 (gcc --version 실행)
fn verify_installation(toolchain_path: &Path) -> Result<String>
// arm-none-eabi-gcc --version 실행 → GCC 버전 문자열 반환
```

### 설치 디렉토리 네이밍

```
~/.embtool/toolchains/
├── nxp-14.2.1/     ← NXP GCC 14.2.1
├── nxp-13.2.1/     ← NXP GCC 13.2.1
└── stm-13.3.1/     ← STM GCC 13.3.1
```

### 의존성
- Unit 0-1 (`paths`)
- Unit 0-2 (`config`)
- Unit 1-1 (`toolchain_registry`)
- Unit 1-2 (`download`)
- Unit 1-3 (`archive`)
- `std::process::Command` (gcc --version)

### 테스트 전략
- list(): 빈 디렉토리 → 빈 목록
- use_version(): config.toml 값 변경 확인
- install(): 통합 테스트 (실제 다운로드는 mock)

---

## Unit 1-5: `embtool toolchain` CLI (`commands/toolchain.rs`)

### 목적
CLI 인자 파싱 → core 함수 호출 → 사용자 친화적 출력

### CLI 정의

```rust
#[derive(Subcommand)]
enum ToolchainAction {
    /// Install a toolchain version
    Install {
        /// Vendor and version (e.g., nxp:14.2, stm:13.3, nxp:latest)
        spec: String,
        /// Force reinstall
        #[arg(long)]
        force: bool,
    },
    /// List installed toolchains
    List {
        /// Also show available (remote) versions
        #[arg(long)]
        available: bool,
    },
    /// Set the active toolchain
    Use {
        /// Vendor:version to activate
        spec: String,
    },
    /// Remove an installed toolchain
    Remove {
        /// Vendor:version to remove
        spec: String,
    },
}
```

### 버전 명세 파싱

```
"nxp:14.2"     → vendor="nxp", version="14.2.1" (매칭)
"stm:13.3"     → vendor="stm", version="13.3.1"
"nxp:latest"   → vendor="nxp", version=latest에서 조회
"14.2"         → vendor 생략 시 embtool.toml의 target.mcu로 추론
                  (Kinetis → nxp, STM32 → stm)
```

### 출력 형식

**install:**
```
📦 Installing NXP ARM GCC 14.2.1...
   Downloading nxp-14.2.1-linux-x64.7z (138 MB)
   [████████████████████████████████] 100% (138/138 MB)
   Verifying SHA256... ✅
📂 Installing to ~/.embtool/toolchains/nxp-14.2.1/
✅ NXP ARM GCC 14.2.1 installed
   arm-none-eabi-gcc (Arm GNU Toolchain 14.2.Rel1) 14.2.1
```

**list:**
```
Installed toolchains:
  * nxp-14.2.1   gcc 14.2.1   (450 MB)   [NXP MCUXpresso]
    nxp-13.2.1   gcc 13.2.1   (480 MB)   [NXP MCUXpresso]
    stm-13.3.1   gcc 13.3.1   (420 MB)   [STM32CubeIDE]
```

**list --available:**
```
Available toolchains:
  Vendor  Version  GCC     Source                    Win   Linux
  ──────  ───────  ──────  ────────────────────────  ────  ─────
  nxp     14.2.1   14.2.1  MCUXpresso IDE 25.6.136  ✅    ✅
  nxp     13.2.1   13.2.1  MCUXpresso IDE            ✅    -
  stm     13.3.1   13.3.1  STM32CubeIDE 1.0.100     ✅    -

Installed: nxp-14.2.1 (active), nxp-13.2.1
```

**use:**
```
🔄 Switched to STM ARM GCC 13.3.1
   arm-none-eabi-gcc 13.3.1
```

**remove:**
```
🗑️  Removed NXP ARM GCC 13.2.1
   Freed 480 MB
```

### 의존성
- Unit 1-4 (`toolchain_manager`)
- `colored`
