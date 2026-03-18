---
name: verify-float-suffix
description: "Checks floating-point literals for uppercase 'F' suffix to prevent double-precision usage on Cortex-M4 soft-float. Use proactively when modifying .c/.h files containing math, ADC, or calibration code. Do NOT use for CMSIS vendor headers or System/ startup code."
user-invokable: false
---

# Verify MCU Float Suffix

## Purpose

Enforces the use of the `F` suffix for floating-point literals.

- **Soft-Float Environment**: MK10D7 (Cortex-M4) has no FPU. Using `double` (e.g., `0.0`) instead of `float` (`0.0F`) severely degrades performance and increases code size.
- **Rule**: `CLAUDE.md` explicitly requires uppercase `F` suffix for float literals.

## When to Run

- Executed automatically by `/check-verify`
- When modifying math calculations, ADC scaling, or calibration code

## Related Files

- `Sources/**/*.c`
- `Sources/**/*.h`
- `Drivers/**/*.c`
- `Drivers/**/*.h`

## Workflow

### Step 1: Detect Missing Suffixes

Search for floating-point literals that lack the `F` or `f` suffix.

```bash
# Finds patterns like 1.0, 0.5, 3.14 that do not end with F/f
# (Excludes double slashes for comments and common hex contexts)
grep -rE "\b[0-9]+\.[0-9]+[^fF0-9eE]" Sources/ Drivers/ | grep -v "//"
```

*Note: The actual RegExp used by verify scripts should be carefully tuned, but conceptually we are looking for decimals without F.*

### Pass/Fail Criteria

- **PASS**: No matches found. All float literals use the `F` suffix.
- **FAIL**: Float literals without `F` suffix are found.

### Remediation

Add the uppercase `F` suffix to the literal.

```c
// Before (FAIL - uses double precision)
float threshold = 0.5;
float result = adc_val * 3.3 / 65535.0;

// After (PASS - uses single precision)
float threshold = 0.5F;
float result = adc_val * 3.3F / 65535.0F;
```

## Exceptions

The following are NOT considered issues:

1. **Comments and Strings**: Decimals inside block/line comments or strings.
2. **Version Numbers**: `v1.0.3` etc.
3. **Double necessity**: Cases where `double` precision is explicitly intended and documented (rare in this MCU).
