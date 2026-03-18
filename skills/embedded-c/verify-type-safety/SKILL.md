---
name: verify-type-safety
description: "Checks for primitive type usage (int, short, long, unsigned char) and enforces stdint.h explicit-width types (uint8_t, int32_t, etc.). ALWAYS use when user asks to find or check: int 검사, unsigned int 찾기, unsigned char 검사, unsigned long, signed char, unsigned short, bare int, 원시 타입 검색, 플랫폼 의존 타입, int 대신 uint32_t, stdint 검사, 타입 검사, 타입 안전성 검사, type check, 타입 점검, 명시적 크기 타입. Use proactively when declaring variables, struct members, or function signatures in .c/.h files. Do NOT use for CMSIS vendor headers, System/ startup code, or main() return type."
user-invokable: false
---

# Verify Type Safety

## Purpose

Ensures type sizes are predictable and architecture-independent.

- **Embedded Safety**: Primitive types like `int` can vary in size (16-bit vs 32-bit) depending on the architecture. ARM Cortex-M4 treats `int` as 32-bit, but explicit typing prevents bugs when porting or interacting with hardware registers.
- **Rule**: Standardized on `stdint.h` (`uint32_t`, `int16_t`, `uint8_t`, etc.).

## When to Run

- Executed automatically by `/check-verify`
- When declaring variables, struct members, function arguments, or return types

## Related Files

- `Sources/**/*.c`
- `Sources/**/*.h`
- `Drivers/**/*.c`
- `Drivers/**/*.h`
- `Components/**/*.c`
- `Components/**/*.h`

## Workflow

### Step 1: Detect Primitive Types

Search for bare primitive variable declarations.

```bash
# Finds instances of 'unsigned int', 'long', 'short', or bare 'int'
# Excludes main() signature and comments
grep -rE "\b(unsigned int|long|short|unsigned char|signed char)\b" Sources/ Drivers/ Components/ | grep -v "//"
grep -rE "\bint\b\s+[a-zA-Z_]" Sources/ Drivers/ Components/ | grep -v "int main" | grep -v "//"
```

### Pass/Fail Criteria

- **PASS**: No matches found. All types use `stdint.h` width-explicit types.
- **FAIL**: Code uses ambiguous primitive types.

### Remediation

Replace primitives with `<stdint.h>` equivalents.

```c
// Before (FAIL)
unsigned int counter = 0;
long delay_time = 1000;
int read_adc(void);

// After (PASS)
uint32_t counter = 0;
int32_t delay_time = 1000;
int32_t read_adc(void);
```

### Step 2: Implicit Narrowing Detection

Detect assignments where a wider integer type is assigned to a narrower type without explicit cast.

**Detection Approach**:

1. Find variable declarations with narrow types: `(uint8_t|int8_t|uint16_t|int16_t)\s+\w+\s*=`
2. For each, trace the right-hand side expression:
   - Function call → check return type in header
   - Variable → check declaration type
   - Arithmetic expression → result type follows C promotion rules (typically int/uint32_t)
3. Flag if RHS type is wider than LHS type AND no explicit cast `(uint16_t)` is present

**Common Narrowing Patterns**:

| Narrowing | Risk | Example |
|-----------|------|---------|
| `uint32_t` → `uint16_t` | Silent truncation above 65535 | `uint16_t val = read_adc_32bit();` |
| `int32_t` → `int16_t` | Sign + magnitude loss | `int16_t temp = calculate_offset();` |
| `uint32_t` → `uint8_t` | Silent truncation above 255 | `uint8_t idx = count % 512;` |
| `int` → `uint8_t` | Sign loss + truncation | `uint8_t status = get_status();` |

**PASS**: No implicit narrowing found, or all narrowing uses explicit cast
**FAIL**: Assignment narrows without explicit cast

**Remediation**:
```c
// Before (FAIL) — implicit narrowing
uint16_t value = read_32bit_register();

// After (PASS) — explicit cast documents intent
uint16_t value = (uint16_t)read_32bit_register();

// Better — add range check before narrowing
uint32_t raw = read_32bit_register();
assert(raw <= UINT16_MAX);
uint16_t value = (uint16_t)raw;
```

## Exceptions

The following are NOT considered issues:

1. **`int main(void)`**: Standard C entry point.
2. **Third-party/Vendor Code**: Code inside `CMSIS/` or specific vendor libraries.
3. **Loop Counters**: Simple `int i` in tightly scoped `for` loops is sometimes acceptable, though `uint32_t i` is preferred.
