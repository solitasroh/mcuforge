# Phase 1: 툴체인 관리

> Week 2-3 · Unit 1-1 ~ 1-5

---

## Unit 1-1: ARM 툴체인 URL 레지스트리 (`core/toolchain_registry.rs`)

### 목적
버전 문자열 → 다운로드 URL 변환, 알려진 버전 목록 관리

### ARM GNU Toolchain URL 패턴

```
기본 URL:
https://developer.arm.com/-/media/Files/downloads/gnu/{version}/binrel/

파일명 패턴:
  Linux x86_64:  arm-gnu-toolchain-{version}-x86_64-arm-none-eabi.tar.xz
  Linux aarch64: arm-gnu-toolchain-{version}-aarch64-arm-none-eabi.tar.xz
  Windows:       arm-gnu-toolchain-{version}-mingw-w64-i686-arm-none-eabi.zip

예시 (13.3.rel1):
  https://developer.arm.com/-/media/Files/downloads/gnu/13.3.rel1/binrel/
    arm-gnu-toolchain-13.3.rel1-x86_64-arm-none-eabi.tar.xz
```

### 데이터 구조

```rust
pub struct ToolchainRelease {
    pub version: String,          // "13.3.rel1" (정규화된 버전)
    pub short_version: String,    // "13.3" (사용자 입력용)
    pub gcc_version: String,      // "13.3.1"
    pub date: String,             // "2024-06"
}
```

### 내장 버전 레지스트리

| 사용자 입력 | 정규화 | GCC 버전 | 날짜 | 상태 |
|------------|--------|----------|------|------|
| `14.2` | `14.2.rel1` | 14.2.1 | 2024-11 | latest |
| `13.3` | `13.3.rel1` | 13.3.1 | 2024-06 | stable |
| `13.2` | `13.2.rel1` | 13.2.1 | 2023-12 | stable |
| `12.3` | `12.3.rel1` | 12.3.1 | 2023-08 | stable |
| `12.2` | `12.2.rel1` | 12.2.1 | 2023-02 | stable |
| `11.3` | `11.3.rel1` | 11.3.1 | 2022-07 | legacy |
| `10.3` | `10.3-2021.10` | 10.3.1 | 2021-10 | legacy |

### 함수 설계

```rust
/// 알려진 모든 버전 반환
pub fn known_versions() -> &'static [ToolchainRelease]

/// 사용자 입력 → 정규화된 버전 (예: "13.3" → "13.3.rel1")
pub fn normalize_version(input: &str) -> Result<String>

/// 버전 + OS → 다운로드 URL 생성
pub fn resolve_url(version: &str) -> Result<String>

/// 버전 + OS → 파일명 생성
pub fn archive_filename(version: &str) -> String

/// 최신 버전 반환
pub fn latest_version() -> &str
```

### 버전 정규화 규칙
```
"13.3"       → "13.3.rel1"
"13.3.rel1"  → "13.3.rel1" (그대로)
"10.3"       → "10.3-2021.10" (특수 패턴)
"latest"     → 최신 버전으로 치환
```

### 의존성
- 없음 (순수 데이터 + 로직)

### 테스트 전략
- 모든 알려진 버전의 URL 생성 검증
- 잘못된 버전 → 에러 확인
- "latest" 키워드 처리 확인

---

## Unit 1-2: 다운로드 엔진 (`utils/download.rs`)

### 목적
HTTP URL에서 파일 다운로드, 진행바 표시, 캐시 관리

### 함수 설계

```rust
/// URL에서 파일 다운로드
/// - show_progress: true면 indicatif 진행바 표시
/// - 캐시 히트 시 다운로드 스킵
/// - 반환: 다운로드된 파일 경로
pub fn download_file(
    url: &str,
    dest_dir: &Path,        // ~/.embtool/cache/
    filename: &str,
    show_progress: bool,
) -> Result<PathBuf>

/// 캐시에 파일 존재하는지 확인
pub fn is_cached(filename: &str) -> bool

/// 캐시 디렉토리 크기 조회 (MB)
pub fn cache_size_mb() -> Result<u64>

/// 캐시 정리
pub fn clear_cache() -> Result<()>
```

### 다운로드 플로우

```
1. dest_dir/filename 이미 존재? → 스킵 ("Using cached ...")
2. reqwest::blocking::get(url)
3. Content-Length 헤더 → 총 크기
4. indicatif::ProgressBar 생성
   - 스타일: [████████████████████████████████] 100% (245/245 MB)
   - CI 모드: 진행바 비활성화 (println으로 대체)
5. 8KB 청크로 읽으며 파일 쓰기 + 진행바 업데이트
6. 다운로드 완료 → 경로 반환
```

### 에러 처리
- 네트워크 오류 → 재시도 1회
- 404 → "Version not found on server"
- 디스크 공간 부족 → 사전 체크 (Content-Length 비교)

### 의존성
- Unit 0-1 (`paths::cache_dir()`)
- `reqwest` (blocking, rustls-tls)
- `indicatif`

### 테스트 전략
- 소형 테스트 파일 다운로드 (httpbin 등)
- 캐시 히트 시 스킵 확인
- 잘못된 URL → 에러 확인

---

## Unit 1-3: 아카이브 해제 (`utils/archive.rs`)

### 목적
다운로드된 아카이브를 툴체인 디렉토리에 해제

### 함수 설계

```rust
/// 아카이브 해제
/// - .tar.xz → tar + xz2 crate
/// - .zip → zip crate
/// - 반환: 해제된 루트 디렉토리 경로
pub fn extract(
    archive_path: &Path,
    dest_dir: &Path,          // ~/.embtool/toolchains/
    show_progress: bool,
) -> Result<PathBuf>
```

### 해제 플로우

```
1. 확장자 판단 (.tar.xz | .zip)
2. tar.xz:
   a. xz2::read::XzDecoder로 감싸기
   b. tar::Archive로 해제
   c. strip_components 로직 (루트 디렉토리 한 단계 제거)
3. zip:
   a. zip::ZipArchive로 열기
   b. 파일별 추출
4. 해제된 디렉토리를 {version}/ 으로 리네임
5. arm-none-eabi-gcc 바이너리 존재 확인 (검증)
```

### 디렉토리 매핑

```
아카이브 내부:                        → 설치 경로:
arm-gnu-toolchain-13.3.rel1-.../      → ~/.embtool/toolchains/13.3.rel1/
├── arm-none-eabi/                       ├── arm-none-eabi/
├── bin/                                 ├── bin/
├── lib/                                 ├── lib/
└── ...                                  └── ...
```

### 의존성
- Unit 0-1 (`paths::toolchains_dir()`)
- `tar`, `xz2`
- `zip` (새로 추가 — Windows용)

### 테스트 전략
- 소형 tar.xz 생성 후 해제 검증
- 해제 후 기대 파일 존재 확인

---

## Unit 1-4: 툴체인 매니저 (`core/toolchain_manager.rs`)

### 목적
install/list/use/remove 핵심 비즈니스 로직

### 데이터 구조

```rust
pub struct InstalledToolchain {
    pub version: String,       // "13.3.rel1"
    pub gcc_version: String,   // "13.3.1 20240614"
    pub path: PathBuf,         // ~/.embtool/toolchains/13.3.rel1/
    pub is_active: bool,
    pub size_mb: u64,
}
```

### 함수 설계

```rust
/// 툴체인 설치
pub fn install(version: &str, force: bool, ci: bool) -> Result<()>
// 1. toolchain_registry::normalize_version()
// 2. 이미 설치됨? → force 아니면 스킵
// 3. toolchain_registry::resolve_url()
// 4. download::download_file() (미러 우선 → fallback)
// 5. archive::extract()
// 6. verify_installation() — gcc --version 실행
// 7. 첫 설치면 config에 default 설정

/// 설치된 툴체인 목록
pub fn list() -> Result<Vec<InstalledToolchain>>
// 1. toolchains_dir() 스캔
// 2. 각 디렉토리에서 bin/arm-none-eabi-gcc --version 실행
// 3. config.default와 비교하여 is_active 설정

/// 기본 툴체인 변경
pub fn use_version(version: &str) -> Result<()>
// 1. 설치 여부 확인
// 2. config.toml의 toolchain.default 업데이트

/// 툴체인 삭제
pub fn remove(version: &str) -> Result<()>
// 1. active 버전이면 경고
// 2. 디렉토리 삭제
// 3. active였으면 다른 버전으로 전환 (또는 None)

/// 특정 버전의 bin/ 경로 반환
pub fn get_toolchain_bin_path(version: &str) -> Result<PathBuf>
// ~/.embtool/toolchains/{version}/bin/

/// 현재 active 버전 반환
pub fn get_active() -> Result<Option<String>>

/// 설치 검증 (gcc --version 실행)
fn verify_installation(version: &str) -> Result<String>
// arm-none-eabi-gcc --version 실행 → GCC 버전 문자열 반환
```

### 미러 다운로드 전략

```
1. config.mirror.enabled == true?
   → mirror URL에서 다운로드 시도
   → 실패 && config.mirror.fallback == true?
     → ARM 공식에서 다운로드
   → 실패 && fallback == false?
     → 에러
2. mirror.enabled == false
   → ARM 공식에서 바로 다운로드
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
        /// Version (e.g., 13.3, 12.2, latest)
        version: String,
        /// Force reinstall
        #[arg(long)]
        force: bool,
    },
    /// List installed toolchains
    List,
    /// Set the active toolchain
    Use {
        /// Version to activate
        version: String,
    },
    /// Remove an installed toolchain
    Remove {
        /// Version to remove
        version: String,
    },
}
```

### 출력 형식

**install:**
```
📦 Installing ARM GNU Toolchain 13.3.rel1...
   Downloading from ARM official...
   [████████████████████████████████] 100% (245 MB)
📂 Installing to ~/.embtool/toolchains/13.3.rel1/
✅ ARM GNU Toolchain 13.3.rel1 installed
   arm-none-eabi-gcc 13.3.1
```

**list:**
```
Installed toolchains:
  * 13.3.rel1   gcc 13.3.1   (512 MB)
    12.2.rel1   gcc 12.2.1   (480 MB)
```

**use:**
```
🔄 Switched to ARM GNU Toolchain 12.2.rel1
   arm-none-eabi-gcc 12.2.1
```

**remove:**
```
🗑️  Removed ARM GNU Toolchain 10.3-2021.10
   Freed 512 MB
```

### 의존성
- Unit 1-4 (`toolchain_manager`)
- `colored`
