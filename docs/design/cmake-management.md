# CMake 버전 관리 설계

> 작성: 2026-03-08 | 상태: 기획안 (리뷰 대기)

---

## 1. 배경 및 문제

### 현재 상황
- 팀원 4~5명이 각자 시스템 CMake 사용 (apt, chocolatey, 수동 설치 등)
- 버전 불일치 → 빌드 실패, 디버깅 시간 낭비
  - 예: CMake 3.20 기능을 CMakeLists.txt에서 사용 → 팀원 시스템에 3.16 설치 → 빌드 에러
  - 예: CI에서 CMake 버전 다름 → 로컬에서 성공/CI에서 실패
- `cmake_minimum_required(VERSION 3.20)`만으로는 **해결 안 됨** — 에러만 뱉고 설치는 못 함

### 목표
- **프로젝트가 CMake 버전을 "선언"하면 embtool이 자동으로 확보**
- 개발자 수동 설치 제거 → `embtool setup` 한 번이면 끝
- 시스템 CMake와 완전 격리 (충돌 없음)
- CI 환경에서도 동일하게 작동

---

## 2. 아키텍처

### 2.1 전체 흐름

```
embtool.toml                    ~/.embtool/
┌─────────────────┐             ┌──────────────────────────┐
│ [cmake]         │             │ cmake/                   │
│ version = "3.28"│──setup──▶  │   3.28.6/                │
│                 │             │     bin/cmake             │
│ [toolchain]     │             │     bin/ctest             │
│ vendor = "nxp"  │             │     bin/cpack             │
│ version = "14.2"│             │   3.20.0/                │
└─────────────────┘             │     bin/cmake             │
                                │ toolchains/              │
                                │   nxp-14.2.1/            │
                                │ cache/                   │
                                └──────────────────────────┘
```

### 2.2 CMake 설치 경로

```
~/.embtool/cmake/{version}/
├── bin/
│   ├── cmake
│   ├── ctest
│   └── cpack
├── share/
│   └── cmake-3.28/
│       └── Modules/
└── doc/
```

### 2.3 embtool.toml 변경

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
version = "3.28"           # 최소 Major.Minor — 자동으로 최신 patch 해석
                           # "3.28" → 3.28.x 중 최신 설치
                           # "3.28.6" → 정확히 3.28.6 설치
```

---

## 3. CMake 배포 소스

### 3.1 공식 GitHub Releases (Primary)

CMake는 Kitware가 GitHub에서 **모든 버전의 바이너리를 공개 배포**합니다.

**URL 패턴** (예측 가능, 공식):
```
https://github.com/Kitware/CMake/releases/download/v{version}/cmake-{version}-{platform}.{ext}
```

| 플랫폼 | 파일명 패턴 | 크기 |
|---------|------------|------|
| Linux x86_64 | `cmake-3.28.6-linux-x86_64.tar.gz` | ~50MB |
| Linux aarch64 | `cmake-3.28.6-linux-aarch64.tar.gz` | ~50MB |
| Windows x86_64 | `cmake-3.28.6-windows-x86_64.zip` | ~45MB |
| Windows i386 | `cmake-3.28.6-windows-i386.zip` | ~42MB |

**장점:**
- 라이선스 제한 없음 (BSD 3-Clause) — 재배포 자유
- URL 패턴 고정 → 별도 registry 불필요
- SHA256 해시 제공 (`cmake-3.28.6-SHA-256.txt`)

### 3.2 R2 미러 (Optional Fallback)

사내 네트워크 속도나 GitHub 접근 제한 환경을 위해:
```
[registry]
cmake_url = "https://pub-25d9755030a54c3280b7a9f68e9bf67c.r2.dev/cmake/"
```

R2에는 자주 쓰는 버전만 미러링 (필요 시).

### 3.3 버전 탐색

CMake는 GitHub API로 릴리스 목록 조회 가능:
```
GET https://api.github.com/repos/Kitware/CMake/releases?per_page=30
```

단, API rate limit 있으므로:
- **기본 전략**: embtool 바이너리에 known versions 목록 내장
- **온라인 전략**: `--available`에서만 API 호출
- **캐시**: `~/.embtool/cache/cmake-versions.json` (TTL 24h)

---

## 4. 명령어 설계

### 4.1 `embtool cmake install [version]`

```bash
# 프로젝트 cmake.version 기준 설치
$ embtool cmake install
 ↓ Installing CMake 3.28.6...
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% (48/48 MB)
 ✓ CMake 3.28.6 installed (48 MB)

# 특정 버전 직접 지정
$ embtool cmake install 3.20.0

# 최신 안정 버전
$ embtool cmake install latest
```

### 4.2 `embtool cmake list`

```bash
$ embtool cmake list

embtool cmake list

▸  cmake-3.28.6       3.28.6    48 MB   cmake:3.28

╭─────────────────────────────────────────────╮
│ Available (top 5)                           │
│                                             │
│    4.0.2       2026-02         52 MB        │
│    3.31.5      2026-01         51 MB        │
│ ✓  3.28.6      2024-05         48 MB        │
│    3.20.6      2023-11         42 MB        │
│    3.16.9      2022-07         38 MB        │
╰─────────────────────────────────────────────╯
```

### 4.3 `embtool cmake remove <version>`

```bash
$ embtool cmake remove 3.20.0
 ✓ Removed CMake 3.20.0 (freed 42 MB)
```

### 4.4 `embtool setup` 통합

`setup`이 **cmake.version도 함께 처리**:

```bash
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
╰──────────────────────────────────────────────────────╯

 ✓ Toolchain nxp-14.2.1 installed
 ✓ CMake 3.28.6 installed                     ← NEW
 ✓ Generated arm-toolchain.cmake

╭──────────────────────────────────────────────────╮
│ Ready                                            │
│                                                  │
│  → Run 'embtool build' to build the project      │
╰──────────────────────────────────────────────────╯
```

---

## 5. 빌드 시스템 통합

### 5.1 `embtool build`가 올바른 cmake를 사용

```bash
$ embtool build

# 내부적으로:
# 1. embtool.toml에서 cmake.version 읽음 → "3.28"
# 2. ~/.embtool/cmake/3.28.6/ 경로 확인
# 3. 해당 cmake로 빌드 실행:
#    ~/.embtool/cmake/3.28.6/bin/cmake -B build -S .
#    ~/.embtool/cmake/3.28.6/bin/cmake --build build
```

**핵심 원칙**: embtool은 시스템 PATH의 cmake를 **절대 사용하지 않음**.
프로젝트에 지정된 버전만 사용.

### 5.2 환경변수 오버라이드

```bash
# 디버깅/테스트용 — 시스템 cmake 강제 사용
EMBTOOL_CMAKE_PATH=/usr/bin/cmake embtool build
```

### 5.3 IDE 통합 (VSCode)

`embtool setup` 시 `.vscode/settings.json`도 생성 가능:

```json
{
  "cmake.cmakePath": "${env:HOME}/.embtool/cmake/3.28.6/bin/cmake",
  "cmake.configureSettings": {
    "CMAKE_TOOLCHAIN_FILE": "${workspaceFolder}/arm-toolchain.cmake"
  }
}
```

→ **이건 Phase 4 (IDE Integration)에서 다룸**

---

## 6. CI 환경 동작

### 6.1 CI 모드

```bash
# CI에서 (GitLab CI / Jenkins)
$ embtool setup --ci

# --ci 플래그 효과:
# 1. 진행바 대신 텍스트 로그
# 2. 캐시 TTL 무시 (항상 최신 manifest 확인)
# 3. 필요시 자동 다운로드 (프롬프트 없음)
```

### 6.2 GitLab CI 예시

```yaml
# .gitlab-ci.yml
build:
  image: ubuntu:22.04
  before_script:
    - curl -fsSL https://github.com/solitasroh/embtool/releases/latest/download/embtool-linux-x64 -o /usr/local/bin/embtool
    - chmod +x /usr/local/bin/embtool
    - embtool setup --ci      # ← cmake + toolchain 자동 설치
  script:
    - embtool build
  cache:
    paths:
      - ~/.embtool/           # cmake + toolchain 캐시
```

---

## 7. 버전 해석 규칙

| embtool.toml 값 | 동작 | 설치 버전 예시 |
|-----------------|------|---------------|
| `"3.28"` | 3.28.x 중 최신 patch | 3.28.6 |
| `"3.28.3"` | 정확히 3.28.3 | 3.28.3 |
| `"3.20"` | 3.20.x 중 최신 | 3.20.6 |
| 미지정 | embtool 기본값 (현재: 3.28) | 3.28.6 |

### 버전 해석 플로우

```
"3.28" → GitHub releases에서 v3.28.* 태그 검색
       → 최신 patch 선택 (3.28.6)
       → 캐시된 versions 목록에서 먼저 확인
       → 없으면 GitHub API 조회
```

---

## 8. 데이터 구조

### 8.1 GlobalConfig 변경

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub toolchain: ToolchainConfig,
    pub cmake: CmakeConfig,           // NEW
    pub registry: RegistryConfig,
    pub mirror: MirrorConfig,
    // ...
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmakeConfig {
    pub default: Option<String>,       // 기본 cmake 버전
    pub github_mirror: Option<String>, // GitHub mirror URL (optional)
}
```

### 8.2 ProjectConfig 변경

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    pub target: ProjectTarget,
    pub toolchain: ProjectToolchain,
    pub cmake: Option<ProjectCmake>,    // NEW
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectCmake {
    pub version: String,  // "3.28" or "3.28.6"
}
```

### 8.3 CMake Version Registry

```rust
pub struct CmakeRelease {
    pub version: String,     // "3.28.6"
    pub date: String,        // "2024-05-09"
    pub assets: HashMap<String, CmakeAsset>,
}

pub struct CmakeAsset {
    pub url: String,         // GitHub download URL
    pub sha256: String,      // 해시
    pub size: u64,
}
```

---

## 9. 구현 계획

### 기존 Phase와의 관계

| 기존 Phase | 내용 | 상태 |
|-----------|------|------|
| Phase 0 | 경로, 설정, 프로젝트 | ✅ 완료 |
| Phase 1 | 툴체인 다운로드/설치 | ✅ 완료 |
| Phase 2 | 프로젝트 생성 | ✅ 완료 |
| **Phase 1.5** | **CMake 관리 (신규)** | 📝 설계 |
| Phase 3 | 빌드 시스템 | ⬜ |
| Phase 4 | IDE 통합 | ⬜ |

### Phase 1.5 세부 Unit

| Unit | 파일 | 내용 | 예상 |
|------|------|------|------|
| 1.5-1 | `core/cmake_registry.rs` | GitHub API 조회, 버전 해석, 캐시 | 2h |
| 1.5-2 | `utils/download.rs` 확장 | tar.gz/zip 다운로드 + SHA256 | 기존 활용 |
| 1.5-3 | `core/cmake_manager.rs` | install/list/remove + 경로 관리 | 2h |
| 1.5-4 | `commands/cmake.rs` | CLI 명령 (`cmake install/list/remove`) | 1h |
| 1.5-5 | `commands/setup.rs` 확장 | setup에 cmake 검사/설치 통합 | 1h |
| 1.5-6 | `core/project.rs` 확장 | embtool.toml `[cmake]` 섹션 파싱 | 30m |
| 1.5-7 | `core/template.rs` 확장 | `embtool new`에서 cmake 버전 포함 | 30m |

**예상 총 소요: ~7시간**

---

## 10. 핵심 설계 결정 포인트

### ❓ 결정 필요

| # | 질문 | 옵션 | 추천 |
|---|------|------|------|
| 1 | CMake 기본 버전? | 3.20 / 3.28 / latest | **3.28** (2024 LTS급, 대부분 기능 커버) |
| 2 | GitHub API rate limit 대응? | (A) known versions 내장 (B) R2에 버전 목록 (C) 토큰 사용 | **(A)** 내장 + 온라인 보충 |
| 3 | tar.gz 해제 방식? | (A) Rust native (flate2+tar) (B) 시스템 tar | **(A)** Rust native — 7z 의존성 불필요 |
| 4 | `embtool new`에 cmake 기본 포함? | (A) 항상 포함 (B) 옵션 | **(A)** 항상 `[cmake]` 섹션 생성 |
| 5 | embtool.toml 미지정 시? | (A) 에러 (B) 기본값 사용 (C) 시스템 cmake fallback | **(B)** 기본값 3.28 |

---

## 11. 툴체인 vs CMake 관리 비교

| 항목 | 툴체인 (ARM GCC) | CMake |
|------|-----------------|-------|
| 소스 | R2 (자체 호스팅) | GitHub Releases (공식) |
| 라이선스 | NXP EULA 제한 | BSD 3-Clause (자유) |
| 아카이브 형식 | 7z | tar.gz (Linux) / zip (Windows) |
| 해제 도구 | 7z 외부 의존 | **Rust native (flate2+tar)** |
| 버전 탐색 | versions.json (자체) | GitHub API / 내장 목록 |
| 크기 | 100~200MB | 40~55MB |
| 플랫폼 | 4 (win/linux × x64/aarch64) | 4+ (win/linux/mac × x64/aarch64) |

**핵심 차이**: CMake는 tar.gz/zip이므로 **7z 의존성 없이 Rust native로 해제 가능** (`flate2` + `tar` 크레이트). 이건 큰 장점 — 처음 설치하는 사용자도 추가 도구 없이 바로 사용 가능.

---

## 12. 보안 고려

| 위협 | 대응 |
|------|------|
| 위변조된 바이너리 | SHA256 검증 (GitHub 제공 해시) |
| 중간자 공격 | HTTPS only (GitHub + R2) |
| GitHub 위장 URL | 하드코딩된 `github.com/Kitware/CMake` 도메인만 허용 |
| 오래된 버전 취약점 | 경고 표시 (version < 3.16) |
