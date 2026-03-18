---
name: verify-safety-c
description: "Enforces MISRA-C:2012 and CERT-C core safety rules: signed integer overflow, array/buffer boundary checks, uninitialized variable usage, unreachable/dead code, pointer arithmetic safety, and division-by-zero detection. Semantic safety layer above verify-barr-c (style). ALWAYS use when user asks about: MISRA-C, CERT-C, signed overflow, 부호 오버플로우, 정수 오버플로우, integer overflow, 배열 경계, array bounds, buffer overflow, 버퍼 오버플로우, 미초기화 변수, uninitialized variable, 도달 불가 코드, unreachable code, dead code, dead store, 포인터 산술, pointer arithmetic, 안전성 검사, safety check, 정적 안전성 분석, 의미적 안전성, division by zero, 0 나누기, NULL 포인터 역참조, NULL dereference, 'measure.c에서 overflow 검사', 'CERT-C 규칙 검증'. ALWAYS use for semantic safety verification on Sources/*.c, Drivers/*.c computation-heavy code (measure.c, calibration.c, hv.c), or before release safety audit. Do NOT use for: BARR-C coding style rules like braces, goto, switch default, float equality (use verify-barr-c), bare int/uint32_t type checks or narrowing cast detection (use verify-type-safety), clang-tidy or clang-format execution (use do-lint), volatile/ISR/critical section review (use check-code-review), or binary/memory analysis (use check-binary-analysis)."
user-invokable: false
---

# C Safety Verification (MISRA-C / CERT-C Core)

## Purpose

Enforces semantic safety rules that prevent undefined behavior and silent data corruption. This is the safety layer above `verify-barr-c` (which covers coding style).

| Layer | Skill | Focus |
|-------|-------|-------|
| Style | verify-barr-c | Braces, keywords, switch/default, float equality |
| Safety | **verify-safety-c** | Overflow, bounds, initialization, reachability, pointer arithmetic |

## When to Run

- Executed automatically by `/check-verify`
- After modifying computation-heavy code (measure.c, calibration.c)
- Before release for safety-critical code paths

## Related Files

| Path Pattern | Description |
|-------------|-------------|
| `Sources/**/*.c` | Application source files |
| `Sources/**/*.h` | Application header files |
| `Drivers/**/*.c` | Driver source files |
| `Drivers/**/*.h` | Driver header files |
| `Components/**/*.c` | Component source files |
| `Components/**/*.h` | Component header files |

## Workflow

### Check 1: Signed Integer Overflow (MISRA-C Rule 12.1 / CERT INT32-C)

Detect arithmetic operations on signed types that may overflow without guards.

**Strategy**:
1. Find arithmetic on signed types: `int32_t`, `int16_t`, `int8_t`
2. Check for: `a + b`, `a - b`, `a * b` without overflow guard
3. Check for: shift operations on signed types (`int32_t x = a << n`)

**Patterns to flag**:
- Signed addition/subtraction without range check before operation
- Signed multiplication without overflow check
- Left shift on signed integer (undefined if result overflows)

**PASS**: All signed arithmetic has overflow protection or is provably safe (constant operands within range)
**FAIL**: Signed arithmetic without overflow guard

**Remediation**:
```c
// Before (violation) — signed overflow is undefined behavior
int32_t result = a + b;

// After (compliant) — check before operation
if ((b > 0 && a > INT32_MAX - b) || (b < 0 && a < INT32_MIN - b))
{
    return ERROR_OVERFLOW;
}
int32_t result = a + b;
```

### Check 2: Array/Buffer Boundary (MISRA-C Rule 18.1 / CERT ARR30-C)

Detect array access without bounds checking.

**Strategy**:
1. Find array subscript operations: `array[index]`
2. Check if `index` is validated against array size before access
3. Check for pointer arithmetic that may go out of bounds

**Patterns to flag**:
- Array access with variable index and no prior bounds check
- `memcpy`/`memset` with size derived from untrusted source without validation
- Loop index used as array index without `< ARRAY_SIZE` guard in loop condition

**PASS**: All array accesses have provable bounds (constant index within range, or runtime check)
**FAIL**: Array access with unchecked index

**Remediation**:
```c
// Before (violation)
uint8_t value = buffer[index];

// After (compliant)
if (index < BUFFER_SIZE)
{
    uint8_t value = buffer[index];
}
```

### Check 3: Uninitialized Variable (MISRA-C Rule 9.1 / CERT EXP33-C)

Detect variables that may be used before initialization.

**Strategy**:
1. Find local variable declarations without initializer
2. Trace code paths from declaration to first use
3. Flag if any path reaches use without assignment

**Patterns to flag**:
- `uint32_t value;` followed by use in a conditional branch where the other branch doesn't assign
- Function-scoped variables declared at top of function, used much later with conditional assignment in between
- Variables assigned only inside `if` but used after `if/else` (where `else` doesn't assign)

**PASS**: All variables are initialized before use on all code paths
**FAIL**: Variable may be uninitialized on some code path

**Remediation**:
```c
// Before (violation)
uint32_t result;
if (condition)
{
    result = compute();
}
use(result);  // uninitialized if !condition

// After (compliant)
uint32_t result = 0;  // safe default
if (condition)
{
    result = compute();
}
use(result);
```

### Check 4: Unreachable Code (MISRA-C Rule 2.1 / CERT MSC12-C)

Detect code that can never execute.

**Strategy**:
1. Find code after unconditional `return`, `break`, `continue`, `goto`
2. Find conditions that are always true or always false (constant expressions)
3. Find `#if 0` blocks (also caught by plan-quality-audit debt scan)

**Patterns to flag**:
- Statements after `return` in same block
- `if (0)` or `if (false)` blocks
- Code after `while (true)` loop with no `break`
- Function parameters validated and returned early, with remaining code unreachable for that parameter range

**PASS**: No unreachable code detected
**FAIL**: Dead code found

**Remediation**: Remove unreachable code or fix the logic that makes it unreachable.

### Check 5: Pointer Arithmetic Safety (MISRA-C Rule 18.4 / CERT ARR37-C)

Detect unsafe pointer arithmetic operations.

**Strategy**:
1. Find pointer increment/decrement: `ptr++`, `ptr--`, `ptr + n`, `ptr - n`
2. Check if the result is validated against the buffer boundaries
3. Find pointer comparison between pointers to different arrays

**Patterns to flag**:
- Pointer arithmetic without corresponding bounds check
- Subtraction of pointers from different arrays (undefined behavior)
- Casting integer to pointer without clear justification (register access is exempt)

**PASS**: All pointer arithmetic is bounded or operates on known-size buffers
**FAIL**: Unbounded pointer arithmetic

**Remediation**:
```c
// Before (violation)
uint8_t* end = buffer + offset;  // offset could exceed buffer size

// After (compliant)
if (offset <= BUFFER_SIZE)
{
    uint8_t* end = buffer + offset;
}
```

## Output Format

```markdown
## C Safety Verification Report

| # | Rule | Description | Status | Issues |
|---|------|-------------|--------|--------|
| 1 | INT-OVERFLOW | Signed integer overflow | PASS/FAIL | N |
| 2 | ARR-BOUNDS | Array/buffer boundary | PASS/FAIL | N |
| 3 | VAR-INIT | Uninitialized variable | PASS/FAIL | N |
| 4 | DEAD-CODE | Unreachable code | PASS/FAIL | N |
| 5 | PTR-ARITH | Pointer arithmetic safety | PASS/FAIL | N |

### Issues Found

| # | Rule | File:Line | Code | Severity | Remediation |
|---|------|-----------|------|----------|-------------|
```

## Exceptions

1. **Register access**: Pointer casts for memory-mapped register access (e.g., `(volatile uint32_t*)0x40000000`) are not violations
2. **CMSIS/vendor code**: Files under `CMSIS/`, `System/` are exempt
3. **Intentional unsigned wrap**: `uint32_t` wrap-around is defined behavior and is not a signed overflow issue
4. **Constant expressions**: Compile-time constant arithmetic that is provably safe (e.g., `sizeof(array) / sizeof(array[0])`)
5. **DMA buffer pointers**: Pointer arithmetic on DMA buffers with known fixed sizes documented in comments
