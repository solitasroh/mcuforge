# Phase 4: 레거시 마이그레이션

> Week 8-9 · Unit 4-1 ~ 4-2

---

## Unit 4-1: CMake 파서 (`core/migrate_parser.rs`)

### 목적
기존 CMakeLists.txt + arm-toolchain.cmake를 분석하여 embtool.toml 생성에 필요한 정보 추출

### 추출 대상

**CMakeLists.txt에서:**

| 패턴 | 추출 값 | embtool.toml 매핑 |
|------|---------|-------------------|
| `project(name)` | 프로젝트명 | `project.name` |
| `-DMK64F12` | MCU define | MCU DB 역조회 → `target.*` |
| `-mcpu=cortex-m4` | CPU 코어 | `target.core` |
| `-mfloat-abi=soft` | FPU 설정 | `target.fpu` |
| `CMAKE_C_STANDARD 99` | C 표준 | `build.c_standard` |
| `-O0` / `-O1` | 최적화 | `build.optimization.*` |
| `LINKER_FILE ...` | 링커 스크립트 | `build.linker_script` |
| `add_compile_options(...)` | 컴파일 플래그 | `build.flags.common` |

**arm-toolchain.cmake에서:**

| 패턴 | 추출 값 | 용도 |
|------|---------|------|
| `ARM_TOOLCHAIN_PATH "C:/Freescale/..."` | 현재 경로 | 경고 출력 |
| `ARM_TOOLCHAIN_ROOT "..."` | 루트 경로 | 버전 추정 |

### 데이터 구조

```rust
pub struct MigrationPlan {
    /// 감지된 프로젝트명
    pub project_name: Option<String>,

    /// 감지된 MCU (DB 역조회 결과)
    pub detected_mcu: Option<&'static McuInfo>,

    /// raw define 문자열 (예: "MK64F12")
    pub raw_mcu_define: Option<String>,

    /// 감지된 코어 (예: "cortex-m4")
    pub detected_core: Option<String>,

    /// 감지된 FPU (예: "soft")
    pub detected_fpu: Option<String>,

    /// 감지된 C 표준 (예: "c99")
    pub detected_c_standard: Option<String>,

    /// 감지된 컴파일 플래그
    pub detected_flags: Vec<String>,

    /// 감지된 define 매크로
    pub detected_defines: Vec<String>,

    /// 링커 스크립트 경로
    pub linker_script: Option<String>,

    /// 기존 툴체인 경로 (경고 표시용)
    pub legacy_toolchain_path: Option<String>,

    /// 분석 신뢰도 (0.0 ~ 1.0)
    pub confidence: f32,

    /// 경고 메시지
    pub warnings: Vec<String>,
}
```

### 함수 설계

```rust
/// 기존 프로젝트 디렉토리 분석
pub fn analyze(dir: &Path) -> Result<MigrationPlan>

/// CMakeLists.txt 분석
fn parse_cmakelists(content: &str) -> PartialPlan

/// arm-toolchain.cmake 분석
fn parse_toolchain_cmake(content: &str) -> PartialPlan

/// MCU define → MCU DB 역조회
fn reverse_lookup_mcu(define: &str) -> Option<&'static McuInfo>
// "MK64F12" → mcu_db::lookup로 전체 스캔 → define 매칭
```

### 신뢰도 계산

```
+0.3: MCU define 감지 성공
+0.2: core 감지 성공
+0.1: FPU 감지 성공
+0.1: C 표준 감지 성공
+0.1: 링커 스크립트 감지 성공
+0.1: 컴파일 플래그 감지 성공
+0.1: 프로젝트명 감지 성공

≥ 0.7: 자동 마이그레이션 권장
< 0.7: 수동 검토 권장 (경고)
```

### 파싱 전략

```rust
// regex 기반 패턴 매칭
// project({name}) → Regex: r"project\((\w+)\)"
// -DMK{...}       → Regex: r"-D(MK\w+)"
// -mcpu=(.+)      → Regex: r"-mcpu=(\S+)"
// -mfloat-abi=(.+)→ Regex: r"-mfloat-abi=(\S+)"
```

### 의존성
- Unit 2-1 (`mcu_db` — 역조회)
- `regex`

### 테스트 전략
- a2750mcu의 실제 CMakeLists.txt로 파싱 테스트
- 각 패턴별 단위 테스트 (정규식)
- 신뢰도 계산 검증

---

## Unit 4-2: `embtool migrate` 명령 (`commands/migrate.rs`)

### CLI 정의

```rust
/// Migrate existing project to embtool management
Migrate {
    /// Skip confirmation prompt
    #[arg(long)]
    yes: bool,

    /// Dry run — analyze only, don't modify files
    #[arg(long)]
    dry_run: bool,
}
```

### 실행 플로우

```
1. 현재 디렉토리에서 CMakeLists.txt 존재 확인
   → 없으면 에러

2. embtool.toml 이미 존재?
   → 있으면 "Already managed by embtool" 경고

3. migrate_parser::analyze() 실행

4. 분석 결과 출력
   - 감지된 MCU, 코어, FPU
   - 감지된 플래그, 링커 스크립트
   - 신뢰도 퍼센트
   - 경고 메시지 (있다면)

5. confidence < 0.7이면 추가 경고

6. dry_run이면 여기서 종료

7. --yes 아니면 확인 프롬프트
   "Proceed with migration? [Y/n]"

8. 마이그레이션 실행:
   a. arm-toolchain.cmake → arm-toolchain.cmake.bak (백업)
   b. embtool.toml 생성 (분석 결과 기반)
   c. arm-toolchain.cmake 재생성 (embtool 버전)
   d. .gitignore에 build/ 추가 (없으면)

9. 검증 안내
   "Run 'embtool setup && embtool build' to verify"
```

### 출력 예시

```
🔍 Analyzing existing project...

   Found: CMakeLists.txt ✓
   Found: arm-toolchain.cmake ✓

   Project:    a2750lm_application
   MCU:        MK64F12 → MK64FN1M0VLL12 (K64)
   Core:       cortex-m4
   FPU:        soft
   C Standard: c99
   Linker:     System/linkerscript.ld
   Toolchain:  /opt/Freescale/KDS_v3/toolchain ⚠️ legacy

   Confidence: 90%

   ⚠️ Legacy toolchain path detected.
      embtool will use ~/.embtool/toolchains/ instead.

📋 Migration plan:
   1. Backup arm-toolchain.cmake → arm-toolchain.cmake.bak
   2. Generate embtool.toml
   3. Generate new arm-toolchain.cmake (embtool managed)

Proceed? [Y/n] y

✅ Migration complete!
   Verify: embtool setup && embtool build
```

### 안전장치
- 기존 파일은 항상 `.bak` 백업
- `--dry-run`으로 미리 확인 가능
- 신뢰도 낮으면 추가 경고
- CI 모드에서는 `--yes` 필수

### 의존성
- Unit 4-1 (`migrate_parser`)
- Unit 0-3 (`project::save`)
- Unit 2-2 (`template::generate_toolchain_cmake`)
- `dialoguer` (확인 프롬프트)
