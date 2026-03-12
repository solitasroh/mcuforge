---
name: verify-type-safety
description: "Checks for primitive type usage (int, short, long, unsigned char) and enforces stdint.h explicit-width types (uint8_t, int32_t, etc.). Use proactively when declaring variables, struct members, or function signatures in .c/.h files. Do NOT use for CMSIS vendor headers, System/ startup code, or main() return type."
user-invokable: false
---

# Verify Type Safety

## Purpose

Ensures type sizes are predictable and architecture-independent.

- **Embedded Safety**: Primitive types like `int` can vary in size (16-bit vs 32-bit) depending on the architecture. ARM Cortex-M4 treats `int` as 32-bit, but explicit typing prevents bugs when porting or interacting with hardware registers.
- **Rule**: Standardized on `stdint.h` (`uint32_t`, `int16_t`, `uint8_t`, etc.).

## When to Run

- Executed automatically by `/verify-implementation`
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

## Exceptions

The following are NOT considered issues:

1. **`int main(void)`**: Standard C entry point.
2. **Third-party/Vendor Code**: Code inside `CMSIS/` or specific vendor libraries.
3. **Loop Counters**: Simple `int i` in tightly scoped `for` loops is sometimes acceptable, though `uint32_t i` is preferred.
