# Phase 3: 빌드 & 플래시

> Week 6-7 · Unit 3-1 ~ 3-4

---

## Unit 3-1: CMake 래퍼 (`core/builder.rs`)

### 목적
CMake configure + build를 래핑하고, 빌드 결과를 파싱하여 리포트

### 데이터 구조

```rust
pub enum BuildProfile {
    Debug,
    Release,
}

pub struct BuildResult {
    pub success: bool,
    pub elf_path: PathBuf,
    pub flash_used: u32,      // bytes
    pub flash_total: u32,     // bytes (MCU 스펙에서)
    pub ram_used: u32,        // bytes
    pub ram_total: u32,       // bytes
    pub build_time_secs: f64,
}
```

### 함수 설계

```rust
/// 프로젝트 빌드
pub fn build(
    project_dir: &Path,
    profile: BuildProfile,
    verbose: bool,
    clean: bool,
) -> Result<BuildResult>

/// arm-none-eabi-size 출력 파싱
fn parse_size_output(output: &str) -> Result<(u32, u32)>
// text + data = Flash used
// data + bss = RAM used
```

### 빌드 플로우

```
1. find_project() → embtool.toml 읽기
2. 툴체인 경로 확인
   → 없으면 "Run 'embtool setup' first"
3. clean == true → rm -rf build/
4. CMake configure:
   cmake -B build
     -DCMAKE_BUILD_TYPE={Debug|Release}
     -DCMAKE_TOOLCHAIN_FILE=arm-toolchain.cmake
5. CMake build:
   cmake --build build -- -j{num_cpus}
6. Size 파싱:
   arm-none-eabi-size build/{name}.elf
7. BuildResult 구성 및 반환
```

### size 출력 파싱 규칙

```
$ arm-none-eabi-size build/app.elf
   text    data     bss     dec     hex filename
  42768    2544    9744   55056    d710 build/app.elf

Flash used = text + data = 42768 + 2544 = 45312
RAM used   = data + bss  = 2544 + 9744  = 12288
```

### 의존성
- Unit 0-3 (`project`)
- Unit 1-4 (`toolchain_manager::get_toolchain_bin_path`)
- `std::process::Command`

### 테스트 전략
- size 출력 문자열 파싱 유닛 테스트
- CMake 호출은 통합 테스트 (실제 프로젝트 필요)

---

## Unit 3-2: 바이너리 변환 (`core/objcopy.rs`)

### 목적
.elf → .bin / .hex 변환 (플래시용)

### 함수 설계

```rust
/// .elf → .bin 변환
pub fn elf_to_bin(elf_path: &Path, toolchain_bin: &Path) -> Result<PathBuf>
// arm-none-eabi-objcopy -O binary input.elf output.bin

/// .elf → .hex 변환
pub fn elf_to_hex(elf_path: &Path, toolchain_bin: &Path) -> Result<PathBuf>
// arm-none-eabi-objcopy -O ihex input.elf output.hex
```

### 출력 경로 규칙
```
input:  build/app.elf
output: build/app.bin
        build/app.hex
```

### 의존성
- `std::process::Command`

---

## Unit 3-3: PEMicro 플래시 (`core/flasher.rs`)

### 목적
PEMicro 디버그 프로브로 .elf 파일을 MCU에 플래시

### 함수 설계

```rust
/// 프로브 감지 및 플래시
pub fn flash(
    elf_path: &Path,
    mcu_define: &str,
    probe: &str,         // "pemicro"
    interface: &str,     // "swd"
) -> Result<()>

/// 연결된 프로브 감지
pub fn detect_probe() -> Result<ProbeInfo>

pub struct ProbeInfo {
    pub name: String,        // "PEMicro Multilink"
    pub serial: String,
    pub interface: String,   // "USB1"
}
```

### 구현 참고

> ⚠️ PEMicro CLI 도구 상세 조사 필요 (Phase 3 시작 시)
>
> 후보:
> - `pegdbserver_console` (PEMicro 공식)
> - `pyocd` (오픈소스, PEMicro 플러그인)
> - `JLinkExe` 대안 검토
>
> Phase 3 시작 시 마스터와 협의하여 결정

### 의존성
- Unit 0-3 (`project`)
- 외부 CLI 도구 (PEMicro)

---

## Unit 3-4: `embtool build` / `embtool flash` CLI

### `embtool build` CLI

```rust
/// Build the current project
Build {
    /// Build profile
    #[arg(long, default_value = "debug")]
    profile: String,     // debug | release

    /// Clean build directory first
    #[arg(long)]
    clean: bool,

    /// Show verbose CMake output
    #[arg(long)]
    verbose: bool,
}
```

### build 출력 예시

```
🔨 Building a2750lm_application (Debug)
   MCU: MK64FN1M0VLL12
   Toolchain: ARM GNU 13.3.rel1

   [CMake Configure] ✅
   [CMake Build]     ████████████████ 100%

✅ Build successful (1.2s)
   Output: build/a2750lm_application.elf
   Flash:  45,312 / 1,048,576 bytes  (4.3%)  ████░░░░░░░░░░░░
   RAM:    12,288 /   262,144 bytes  (4.7%)  ████░░░░░░░░░░░░
```

### `embtool flash` CLI

```rust
/// Flash firmware to target MCU
Flash {
    /// ELF file to flash (default: build output)
    #[arg(long)]
    elf: Option<String>,
}
```

### flash 출력 예시

```
🔌 Detecting probe... PEMicro Multilink (USB1)
📤 Flashing a2750lm_application.elf (45,312 bytes)
   [████████████████████████████████] 100%
✅ Flash complete
```

### 의존성
- Unit 3-1, 3-2, 3-3
- `colored`, `indicatif`
