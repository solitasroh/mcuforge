---
name: verify-driver-structure
description: "Checks that Drivers/ files never include Sources/ headers (no reverse dependencies) and that driver init functions follow _init naming convention. ALWAYS use when user asks about: 역참조 위반, 역참조 검사, reverse dependency, Drivers include Sources, Drivers/ 의존성 검사, 계층 위반, layer violation, _init 함수 확인, init 명명 규칙, 드라이버 구조 검사, 드라이버 구조 검증, 드라이버 의존성, 애플리케이션 헤더 include, #include Sources 패턴. Use when creating or modifying peripheral drivers in Drivers/. Do NOT use for Sources/ application logic or Components/ middleware."
user-invokable: false
---

# Verify Driver Structure

## Purpose

Maintains clean architectural boundaries within the project.

- **Dependency Rule**: Code in `Drivers/` must only depend on hardware registers (CMSIS) or standard libraries. It must NEVER `#include` headers from `Sources/` (Application Layer) or `Components/` (Middleware).
- **Naming Rule**: Driver initialization functions must be suffixed with `_init` (e.g., `adc_init`, `uart_init`).

## When to Run

- Executed automatically by `/check-verify`
- When creating or modifying a peripheral driver

## Related Files

- `Drivers/**/*.c`
- `Drivers/**/*.h`

## Workflow

### Step 1: Detect Dependency Inversion

Search for illegal `#include` statements within the `Drivers/` directory.

```bash
# Finds any inclusion of application-level headers within driver code
grep -rE '#include\s+["<](Sources|Components)/' Drivers/
# Finds includes that might be relative but point to application files
grep -rE '#include\s+["<](measure|insulation_index|calibration|module_management|hv|dio)\.h[">]' Drivers/
```

### Step 2: Detect Missing Init Functions

Check if each `.c` file in `Drivers/` has a corresponding `_init` function.

```bash
# Extracts filenames and checks if a matching _init function exists
for file in Drivers/*.c; do
    basename=$(basename "$file" .c)
    if ! grep -q "${basename}_init" "$file"; then
        echo "Missing init function in $file: Expected ${basename}_init()"
    fi
done
```

### Pass/Fail Criteria

- **PASS**: No illegal includes found, and all drivers have an `_init` function.
- **FAIL**: Drivers depend on higher-level components or lack standard initialization.

### Remediation

1. **Dependency Issue**: Move the application-specific logic out of the driver and into `Sources/`. Pass data to the driver via function arguments instead of the driver reading application state directly.
2. **Init Issue**: Implement a `void <module>_init(void)` function that handles clock gating (`SIM->SCGCx`) and pin multiplexing (`PORTx->PCR`).

## Exceptions

The following are NOT considered issues:

1. **Utility Headers**: If a header in `Components/` contains purely generic macros or types (e.g., `common_types.h`) and is explicitly whitelisted.
