---
name: do-lint
description: "Runs static analysis (clang-tidy or manual pattern checks) and code formatting (clang-format) on firmware source code. User-invoked broad analysis across directories — unlike verify-* skills which auto-trigger on narrow per-file checks. ALWAYS use this skill when the user mentions: lint, 정적 분석, static analysis, clang-tidy, clang-format, 코드 포맷팅, code formatting, magic number, unused variable, missing static, missing const, 코드 스타일 점검, PR 전 포맷 확인, batch formatting, or wants broad code quality checks across multiple files. Do NOT use for runtime debugging, hardware-specific safety review (use check-code-review --safety), or single-file verify checks (use verify-float-suffix, verify-type-safety, verify-barr-c)."
argument-hint: "<lint|format> [file] [--fix] [--check]"
user-invokable: true
---

# MCU Quality (Lint + Format)

Unified skill combining static analysis and code formatting.

## Sub-commands

### `lint` — Static Analysis

Arguments: `[file]`, `--fix`

#### Strategy Selection

1. **clang-tidy available**: If `clang-tidy` is found on PATH and `compile_commands.json` exists in `output/`, use clang-tidy with embedded-relevant checks.
2. **Manual analysis (fallback)**: Read source files directly and perform pattern-based analysis.

#### Checks Performed

**Critical (Embedded Safety)**:
- **volatile missing**: Variables accessed in both ISR and main context without `volatile`
- **ISR safety**: Use of non-reentrant functions (malloc, printf, etc.) in interrupt handlers
- **Stack overflow risk**: Large local arrays or deep recursion in ISR context
- **Integer overflow**: Arithmetic on unsigned types near boundary values
- **Null pointer dereference**: Pointer use without null checks at module boundaries

**Important (Code Quality)**:
- **Type safety**: Use of `int`/`short`/`long` instead of `stdint.h` types
- **Magic numbers**: Hardcoded numeric values that should be `#define` or `enum`
- **Missing `const`**: Pointer parameters that could be `const`
- **Unused variables**: Declared but unused variables
- **Missing `static`**: File-scope functions/variables that should be `static`

**Style (Coding Standards)**:
- **Naming convention**: Functions and variables should use `snake_case` with module prefix
- **Header guards**: Missing or incorrect include guards (see Header Guard Consistency below)
- **Include order**: System headers before project headers

#### Header Guard Consistency

Check all `.h` files for proper include guards:

1. **Pattern**: Every `.h` file must have `#ifndef FILENAME_H` / `#define FILENAME_H` / `#endif` or `#pragma once`
2. **Naming**: Guard macro must be `UPPER_SNAKE_CASE` + `_H` suffix matching the filename
   - `measure.h` → `#ifndef MEASURE_H`
   - `adc_gathering.h` → `#ifndef ADC_GATHERING_H`
3. **Detection**:
   - Grep for `.h` files without `#ifndef` in first 10 lines AND without `#pragma once`
   - Check that guard name matches filename
4. **Exceptions**: CMSIS vendor headers, third-party headers

```
[STYLE] Sources/new_module.h:1 - Missing header guard (#ifndef NEW_MODULE_H)
[STYLE] Drivers/gpio.h:2 - Header guard name mismatch: GPIO_DRIVER_H vs expected GPIO_H
```

#### Source Locations

- `./Sources/` — Application source code
- `./Components/` — Only if specifically requested

#### Output Format

```
[CRITICAL] Sources/main.c:42 - Variable 'adc_value' accessed in ISR without volatile qualifier
[IMPORTANT] Sources/comm.c:128 - Use uint16_t instead of unsigned short
[STYLE] Sources/init.c:15 - Function 'init_hardware' missing module prefix
```

### `format` — Code Formatting

Arguments: `[file]`, `--check`, `--fix`

1. **Locate .clang-format**: Search for `.clang-format` in project root.

2. **Find source files**: Collect all `.c` and `.h` files in `./Sources/` (default) or specified path.

3. **Check mode** (default or `--check`):
   - Run `clang-format --dry-run --Werror` on each file
   - Report files that need formatting: `3 of 15 files need formatting`

4. **Fix mode** (`--fix`):
   - Run `clang-format -i` on each file
   - Report total number of files formatted

5. **No clang-format**: If not on PATH, report the issue and suggest installing. Do NOT manually reformat.
