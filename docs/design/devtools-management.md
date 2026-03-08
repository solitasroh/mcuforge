# 개발 도구 통합 관리 설계

> 작성: 2026-03-08 | 상태: 기획안 (리뷰 대기)
> 이전 문서 `cmake-management.md`를 확장하여 통합

---

## 1. 개요

### 목표
`embtool`이 **빌드 환경의 모든 외부 도구를 선언적으로 관리**한다.

```toml
# embtool.toml — 이것만 선언하면 끝
[toolchain]
vendor = "nxp"
version = "14.2.1"

[cmake]
version = "3.28"

[tools.clang-format]
version = "18"

[tools.clang-tidy]
version = "18"
```

```bash
embtool setup     # ← 한 번이면 모든 도구 확보
```

### 관리 대상

| 도구 | 용도 | 소스 | 크기 |
|------|------|------|------|
| **ARM GCC** | 크로스 컴파일러 | R2 (자체 호스팅) | 100~200MB |
| **CMake** | 빌드 시스템 | GitHub (Kitware/CMake) | 40~55MB |
| **clang-format** | 코드 포맷팅 | GitHub (cpp-linter/clang-tools-static-binaries) | 2~4MB |
| **clang-tidy** | 정적 분석 | GitHub (cpp-linter/clang-tools-static-binaries) | 40~76MB |

---

## 2. 배포 소스 상세

### 2.1 CMake — Kitware 공식 GitHub Releases

| 항목 | 값 |
|------|-----|
| 리포 | `github.com/Kitware/CMake` |
| 라이선스 | BSD 3-Clause (재배포 자유) |
| 형식 | tar.gz (Linux/macOS), zip (Windows) |
| Rust 해제 | `flate2` + `tar` / `zip` 크레이트 (7z 불필요) |

**URL 패턴:**
```
https://github.com/Kitware/CMake/releases/download/v{version}/cmake-{version}-{platform}.{ext}
```

**플랫폼 매핑:**

| embtool 플랫폼 | CMake 파일명 | 크기 |
|----------------|-------------|------|
| `linux-x64` | `cmake-{v}-linux-x86_64.tar.gz` | ~50MB |
| `linux-aarch64` | `cmake-{v}-linux-aarch64.tar.gz` | ~50MB |
| `win-x64` | `cmake-{v}-windows-x86_64.zip` | ~45MB |
| `macos-universal` | `cmake-{v}-macos-universal.tar.gz` | ~55MB |

**지원 버전:** 3.16+ (2019~현재, 모든 릴리스)

### 2.2 clang-format / clang-tidy — Static Binaries

| 항목 | 값 |
|------|-----|
| 리포 | `github.com/cpp-linter/clang-tools-static-binaries` |
| 라이선스 | Apache 2.0 (재배포 자유) |
| 형식 | **단일 바이너리** (아카이브 해제 불필요!) |
| 체크섬 | SHA512 (`{file}.sha512sum`) |

**URL 패턴:**
```
https://github.com/cpp-linter/clang-tools-static-binaries/releases/download/{tag}/{tool}-{version}_{platform}
```

**핵심 장점: 단일 실행 파일 → 다운로드 즉시 사용 가능 (해제 없음)**

**플랫폼 매핑:**

| embtool 플랫폼 | clang-tools 파일명 | clang-format | clang-tidy |
|----------------|-------------------|-------------|-----------|
| `linux-x64` | `{tool}-{v}_linux-amd64` | 2~4MB | 34~76MB |
| `win-x64` | `{tool}-{v}_windows-amd64.exe` | 1~2MB | 36~68MB |
| `macos-x64` | `{tool}-{v}_macos-intel-amd64` | 2~4MB | 36~74MB |
| `macos-arm64` | `{tool}-{v}_macosx-arm64` | 2~4MB | 34~72MB |

**지원 버전:** 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22
**모든 플랫폼 (Linux/Windows/macOS) × 모든 버전 지원 확인됨** ✅

### 2.3 크기 비교 (전체 설치 시)

```
전체 embtool 도구 셋 (최소):
  ARM GCC toolchain    138MB  (nxp-14.2.1 linux-x64)
  CMake 3.28            50MB
  clang-format 18        4MB
  clang-tidy 18         65MB
  ─────────────────────────
  합계                 ~257MB

전체 LLVM 설치 시:
  LLVM 22             1850MB  ← 7.2배 더 큼
```

---

## 3. 설치 경로

```
~/.embtool/
├── toolchains/
│   ├── nxp-14.2.1/           # ARM GCC (기존)
│   └── stm-13.3.1/
├── cmake/
│   ├── 3.28.6/               # CMake (NEW)
│   │   └── bin/cmake
│   └── 3.20.6/
├── tools/
│   ├── clang-format/         # clang-format (NEW)
│   │   ├── 18/
│   │   │   └── clang-format  # 단일 바이너리
│   │   └── 22/
│   │       └── clang-format
│   └── clang-tidy/           # clang-tidy (NEW)
│       ├── 18/
│       │   └── clang-tidy
│       └── 22/
│           └── clang-tidy
└── cache/
    ├── cmake-3.28.6-linux-x86_64.tar.gz
    └── versions.json
```

---

## 4. embtool.toml 설계

### 4.1 전체 형태

```toml
[project]
name = "a2750lm_application"
type = "application"

[target]
mcu = "MK64F12"
core = "cortex-m4"

[toolchain]
vendor = "nxp"
version = "14.2.1"

[cmake]
version = "3.28"              # Major.Minor → 최신 patch 자동

[tools.clang-format]
version = "18"                # Major만 → 해당 메이저 바이너리
config = ".clang-format"      # 포맷 설정 파일 (optional)

[tools.clang-tidy]
version = "18"
config = ".clang-tidy"        # 분석 설정 파일 (optional)
```

### 4.2 섹션 설명

| 섹션 | 필수 | 설명 |
|------|------|------|
| `[toolchain]` | ✅ | ARM GCC 크로스 컴파일러 |
| `[cmake]` | ❌ | 미지정 시 기본 3.28 |
| `[tools.clang-format]` | ❌ | 미지정 시 설치 안 함 |
| `[tools.clang-tidy]` | ❌ | 미지정 시 설치 안 함 |

---

## 5. 명령어 설계

### 5.1 통합 `embtool setup`

```
$ embtool setup

embtool setup
  /home/user/projects/a2750lm_application

╭──────────────────────────────────────────────────────╮
│ Project                                              │
│                                                      │
│   Name              a2750lm_application              │
│   MCU               MK64F12 (cortex-m4)              │
│   Toolchain         nxp:14.2.1                       │
│   CMake             3.28                              │
│   clang-format      18                               │
│   clang-tidy        18                               │
╰──────────────────────────────────────────────────────╯

 ✓ Toolchain nxp-14.2.1 installed
 ✓ CMake 3.28.6 installed
 ✓ clang-format 18 installed
 ✓ clang-tidy 18 installed
 ✓ Generated arm-toolchain.cmake

╭───────────────────────────────────────────╮
│ Ready                                     │
│                                           │
│  → Run 'embtool build' to build           │
╰───────────────────────────────────────────╯
```

### 5.2 개별 도구 관리

```bash
# CMake
embtool cmake install [version]
embtool cmake list [--available]
embtool cmake remove <version>

# clang-format
embtool tool install clang-format [version]
embtool tool list clang-format [--available]
embtool tool remove clang-format <version>

# clang-tidy
embtool tool install clang-tidy [version]
embtool tool list clang-tidy [--available]
embtool tool remove clang-tidy <version>

# 전체 도구 상태
embtool tool list
```

### 5.3 `embtool tool list` 출력

```
embtool tool list

╭──────────────────────────────────────────────────────╮
│ Installed Tools                                      │
│                                                      │
│ ▸  cmake           3.28.6         50 MB   cmake:3.28 │
│ ▸  clang-format    18             4 MB    clang:18   │
│ ▸  clang-tidy      18             65 MB   clang:18   │
╰──────────────────────────────────────────────────────╯
```

### 5.4 `embtool tool list --available`

```
╭─────────────────────────────────────────────────╮
│ clang-format — Available for linux-x64          │
│                                                 │
│ ✓  22          4 MB                             │
│    21          4 MB                             │
│    20          4 MB                             │
│    19          4 MB                             │
│ ✓  18          4 MB                             │
│    17          3 MB                             │
│    16          3 MB                             │
│    15          3 MB                             │
│    14          3 MB                             │
│    13          2 MB                             │
│    12          2 MB                             │
│    11          2 MB                             │
╰─────────────────────────────────────────────────╯
```

---

## 6. 빌드 통합

### 6.1 `embtool build`

```bash
$ embtool build

# 내부적으로:
# 1. embtool.toml 읽기
# 2. cmake → ~/.embtool/cmake/3.28.6/bin/cmake
# 3. toolchain → ~/.embtool/toolchains/nxp-14.2.1/
# 4. 실행:
#    $CMAKE -B build -S . -DCMAKE_TOOLCHAIN_FILE=arm-toolchain.cmake
#    $CMAKE --build build
```

### 6.2 `embtool format`

```bash
$ embtool format [--check] [paths...]

# 내부적으로:
# 1. embtool.toml → tools.clang-format.version = "18"
# 2. ~/.embtool/tools/clang-format/18/clang-format
# 3. 실행:
#    $CLANG_FORMAT -i --style=file src/**/*.c src/**/*.h
#
# --check: CI용, 변경이 필요하면 exit 1
```

### 6.3 `embtool lint`

```bash
$ embtool lint [paths...]

# 내부적으로:
# 1. embtool.toml → tools.clang-tidy.version = "18"
# 2. ~/.embtool/tools/clang-tidy/18/clang-tidy
# 3. 실행:
#    $CLANG_TIDY -p build src/**/*.c
#    (compile_commands.json from build/)
```

---

## 7. 플랫폼 지원 매트릭스

### 7.1 OS 감지

```rust
pub fn platform_triple() -> (&'static str, &'static str) {
    // (os, arch)
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { ("linux", "x64") }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { ("linux", "aarch64") }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { ("windows", "x64") }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { ("macos", "x64") }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { ("macos", "arm64") }
}
```

### 7.2 도구별 플랫폼 매핑

```rust
trait ToolProvider {
    fn name(&self) -> &str;
    fn download_url(&self, version: &str, os: &str, arch: &str) -> Result<String>;
    fn archive_type(&self) -> ArchiveType;
    fn verify_install(&self, path: &Path) -> Result<String>;  // returns version
}

enum ArchiveType {
    TarGz,      // CMake (Linux/macOS)
    Zip,        // CMake (Windows)
    SingleBinary, // clang-format, clang-tidy
}
```

| 도구 | linux-x64 | linux-aarch64 | win-x64 | macos-x64 | macos-arm64 |
|------|-----------|--------------|---------|-----------|-------------|
| ARM GCC | ✅ R2 | ❌ | ✅ R2 | ❌ | ❌ |
| CMake | ✅ tar.gz | ✅ tar.gz | ✅ zip | ✅ tar.gz | ✅ tar.gz |
| clang-format | ✅ binary | ❌ * | ✅ exe | ✅ binary | ✅ binary |
| clang-tidy | ✅ binary | ❌ * | ✅ exe | ✅ binary | ✅ binary |

\* linux-aarch64: clang-tools-static-binaries가 amd64만 제공. 향후 R2 미러 가능.

---

## 8. 아카이브 처리 — 도구별 전략

| 도구 | 형식 | 해제 방법 | 외부 의존성 |
|------|------|----------|-----------|
| ARM GCC | 7z | 시스템 `7z` | ⚠️ 필요 |
| CMake | tar.gz / zip | **Rust native** (`flate2`+`tar` / `zip`) | ❌ 없음 |
| clang-format | **단일 바이너리** | 다운로드 → chmod +x | ❌ 없음 |
| clang-tidy | **단일 바이너리** | 다운로드 → chmod +x | ❌ 없음 |

**clang-format/clang-tidy는 아카이브 해제가 아예 필요 없습니다.**
가장 가벼운 설치 경험.

---

## 9. 버전 해석 규칙

### CMake

| 입력 | 해석 | 예시 |
|------|------|------|
| `"3.28"` | 3.28.x 중 최신 patch | → 3.28.6 |
| `"3.28.3"` | 정확히 3.28.3 | → 3.28.3 |
| `"latest"` | 최신 안정 릴리스 | → 4.0.2 |
| 미지정 | 기본값 3.28 | → 3.28.6 |

### clang-format / clang-tidy

| 입력 | 해석 | 예시 |
|------|------|------|
| `"18"` | 메이저 버전 18 | → 18 바이너리 |
| `"22"` | 메이저 버전 22 | → 22 바이너리 |
| `"latest"` | 최신 | → 22 |
| 미지정 | 설치 안 함 | — |

clang-tools-static-binaries는 **메이저 버전 단위**로 배포 (11~22).

---

## 10. CI 환경

### GitLab CI 예시

```yaml
build:
  image: ubuntu:22.04
  before_script:
    - curl -fsSL https://github.com/solitasroh/embtool/releases/latest/download/embtool-linux-x64 -o /usr/local/bin/embtool
    - chmod +x /usr/local/bin/embtool
    - embtool setup --ci
  script:
    - embtool format --check           # 코드 포맷 검사
    - embtool lint                     # 정적 분석
    - embtool build                    # 빌드
  cache:
    paths:
      - ~/.embtool/                    # 모든 도구 캐시
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any
    stages {
        stage('Setup') {
            steps {
                sh 'embtool setup --ci'
            }
        }
        stage('Quality') {
            parallel {
                stage('Format') { steps { sh 'embtool format --check' } }
                stage('Lint')   { steps { sh 'embtool lint' } }
            }
        }
        stage('Build') {
            steps { sh 'embtool build' }
        }
    }
}
```

---

## 11. 구현 계획

### Phase 구조 변경

| Phase | 내용 | 상태 | 비고 |
|-------|------|------|------|
| Phase 0 | 경로, 설정, 프로젝트 | ✅ | |
| Phase 1 | ARM GCC 다운로드/설치 | ✅ | |
| Phase 2 | 프로젝트 생성 | ✅ | |
| **Phase 1.5** | **개발 도구 관리** | 📝 | **CMake + clang-format + clang-tidy** |
| Phase 3 | 빌드 시스템 | ⬜ | `embtool build` + `embtool format` + `embtool lint` |
| Phase 4 | IDE 통합 | ⬜ | |

### Phase 1.5 구현 Unit

| Unit | 파일 | 내용 | 예상 |
|------|------|------|------|
| 1.5-1 | `core/tool_provider.rs` | `ToolProvider` trait, `ArchiveType`, 플랫폼 매핑 | 2h |
| 1.5-2 | `core/cmake_provider.rs` | CMake: GitHub URL 빌드, version 해석, tar.gz/zip 해제 | 3h |
| 1.5-3 | `core/clang_provider.rs` | clang-format/tidy: Static binary URL, SHA512, single file install | 2h |
| 1.5-4 | `core/tool_manager.rs` | install/list/remove 통합 (trait 기반) | 2h |
| 1.5-5 | `commands/tool.rs` | `embtool tool install/list/remove` CLI | 1h |
| 1.5-6 | `commands/cmake.rs` | `embtool cmake install/list/remove` CLI | 1h |
| 1.5-7 | `commands/setup.rs` 확장 | setup에서 cmake + tools 통합 확인/설치 | 1h |
| 1.5-8 | `core/project.rs` 확장 | embtool.toml `[cmake]`, `[tools.*]` 파싱 | 1h |
| 1.5-9 | `core/template.rs` 확장 | `embtool new`에 cmake/tools 버전 포함 | 30m |
| 1.5-10 | Cargo.toml | `flate2`, `tar`, `zip` 크레이트 추가 | 10m |

**예상 총 소요: ~14시간**

---

## 12. Cargo.toml 추가 의존성

```toml
[dependencies]
# 기존
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
sha2 = "0.10"
reqwest = { version = "0.12", features = ["blocking", "rustls-tls"] }
indicatif = "0.17"
iocraft = "0.7"

# Phase 1.5 추가
flate2 = "1"          # gzip 해제 (CMake tar.gz)
tar = "0.4"           # tar 해제 (CMake)
zip = "2"             # zip 해제 (CMake Windows)
sha2 = "0.10"         # SHA256 (기존) + SHA512도 지원
```

---

## 13. 결정 필요 사항

| # | 질문 | 추천 | 이유 |
|---|------|------|------|
| 1 | CMake 기본 버전 | **3.28** | 2024 LTS급, FILE_SET 지원, presets 완성 |
| 2 | clang 기본 버전 (미지정 시) | **설치 안 함** | opt-in — 필요한 프로젝트만 |
| 3 | cmake 서브커맨드 vs tool 통합 | **cmake 별도** | cmake는 빌드 필수, tools는 선택 |
| 4 | `embtool new` 시 기본 포함 | CMake ✅, clang ❌ | cmake는 기본, clang은 opt-in |
| 5 | `embtool format`/`lint` 커맨드 | **Phase 3에서** | 설치는 1.5, 실행은 3 |
| 6 | R2 미러 필요? | **나중에** | GitHub 직접 접근이 우선 |

---

## 14. `embtool new` 변경

```bash
# 기본: cmake 포함, clang 미포함
$ embtool new my_app --mcu k64

# clang 도구 포함
$ embtool new my_app --mcu k64 --with-clang 18

# 결과 embtool.toml
[cmake]
version = "3.28"

[tools.clang-format]
version = "18"

[tools.clang-tidy]
version = "18"
```

### 추가 생성 파일

| 파일 | 조건 | 내용 |
|------|------|------|
| `.clang-format` | `--with-clang` | 기본 포맷 규칙 (LLVM 기반, 커스텀) |
| `.clang-tidy` | `--with-clang` | 기본 검사 규칙 (embedded 최적화) |

---

## 15. 보안

| 위협 | 대응 |
|------|------|
| 바이너리 위변조 | SHA256 (CMake) / SHA512 (clang-tools) 검증 |
| MITM | HTTPS only |
| 신뢰할 수 없는 소스 | 하드코딩된 GitHub 도메인만 허용 |
| 실행 권한 | Linux/macOS: chmod +x, Windows: .exe |
