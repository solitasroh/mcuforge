---
name: mcu-quality
description: "Runs static analysis (clang-tidy or manual pattern checks) and code formatting (clang-format) on firmware source code. Use for lint checks, style enforcement, or batch formatting. Do NOT use for runtime debugging or hardware-specific safety review."
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
- **Header guards**: Missing or incorrect include guards
- **Include order**: System headers before project headers

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
