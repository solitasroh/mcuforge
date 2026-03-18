---
name: verify-barr-c
description: "Enforces BARR-C:2018 high-priority coding rules: braces on all if/for/while, no goto/auto/register, no assignment in conditions, switch requires default, no float equality. ALWAYS use when user asks about: goto 검사, goto 사용, goto 찾기, 조건문 대입 검사, 조건문 안에 대입, 중괄호 검사, 중괄호 빠진 곳, braces 검사, switch default 검사, switch default 빠진 곳, float == 비교, float 동등 비교, auto 키워드, register 키워드, BARR-C, BARR-C 규칙, 코딩 규칙 위반 검사. Use proactively when modifying .c/.h files in Sources/ or Drivers/. Do NOT use for CMSIS vendor headers or System/ startup code."
user-invokable: false
---

# BARR-C:2018 Coding Rules Verification

## Purpose

Enforces high-priority BARR-C:2018 rules that prevent common embedded bugs:

1. **Rule 1.3** - Braces required on all if/for/while/do bodies
2. **Rule 1.7** - Forbidden keywords: `goto`, `auto`, `register`
3. **Rule 8.2b** - No assignment inside conditions
4. **Rule 8.3** - Every `switch` must have a `default` case
5. **Rule 5.4** - No float/double equality comparison (`==`, `!=`)
6. **NULL Guard** - Public functions with pointer params must check NULL at entry
7. **Rule 8.3+** - Enum switch must have explicit `case` for every enum value

## When to Run

- After modifying `.c` or `.h` files in Sources/ or Drivers/
- Before PR to ensure BARR-C compliance
- When check-verify orchestrates verification

## Related Files

| Path Pattern | Description |
|-------------|-------------|
| `Sources/**/*.c` | Application source files |
| `Sources/**/*.h` | Application header files |
| `Drivers/**/*.c` | Driver source files |
| `Drivers/**/*.h` | Driver header files |

## Workflow

### Check 1: Braces on Control Statements (Rule 1.3)

Detect `if`, `else`, `for`, `while`, `do` followed by a statement without opening brace.

**Tool**: Grep
**Pattern**: `(if|else|for|while)\s*\(.*\)\s*$` (line ends after closing paren — next line is body without brace)
**Path**: `Sources/`, `Drivers/`
**Glob**: `*.c`

Also check single-line form:
**Pattern**: `(if|else if|for|while)\s*\(.*\)\s+[^{/\s]`

**PASS**: No matches found
**FAIL**: Match found — wrap body in `{ }`

**Remediation**:
```c
// Before (violation)
if (x > 0)
    do_something();

// After (compliant)
if (x > 0)
{
    do_something();
}
```

### Check 2: Forbidden Keywords (Rule 1.7)

Detect `goto`, `auto`, `register` keywords.

**Tool**: Grep
**Pattern**: `\b(goto|auto|register)\b`
**Path**: `Sources/`, `Drivers/`
**Glob**: `*.{c,h}`

**PASS**: No matches found
**FAIL**: Match found — remove keyword or refactor

**Remediation**:
- `goto` → restructure with loops/flags
- `register` → remove (compiler optimizes automatically)
- `auto` → use explicit type

### Check 3: Assignment in Conditions (Rule 8.2b)

Detect assignment (`=`) inside `if`/`while` condition expressions.

**Tool**: Grep
**Pattern**: `(if|while)\s*\([^)]*[^!=<>]=(?!=)[^=]`
**Path**: `Sources/`, `Drivers/`
**Glob**: `*.c`

**PASS**: No matches found
**FAIL**: Match found — separate assignment from condition

**Remediation**:
```c
// Before (violation)
if (result = read_value())

// After (compliant)
result = read_value();
if (result)
```

### Check 4: Switch Default Case (Rule 8.3)

Find all `switch` blocks and verify each has a `default` case.

**Tool**: Grep (multiline)
**Pattern**: `switch\s*\([^)]+\)\s*\{` to find switch blocks
**Path**: `Sources/`, `Drivers/`
**Glob**: `*.c`

For each match, read the enclosing block and verify `default:` exists.

**PASS**: All switch blocks contain `default:`
**FAIL**: Switch block missing `default:` — add `default: break;` or `default: /* Do nothing */ break;`

**Remediation**:
```c
switch (state)
{
    case STATE_IDLE:
        break;
    case STATE_RUN:
        break;
    default:
        break;
}
```

### Check 5: Float Equality Comparison (Rule 5.4)

Detect direct equality comparison with floating-point values.

**Tool**: Grep
**Pattern**: `[!=]=\s*[0-9]+\.[0-9]|[0-9]+\.[0-9]+[fF]?\s*[!=]=`
**Path**: `Sources/`, `Drivers/`
**Glob**: `*.c`

Also check variable-to-variable float comparison in context (requires reading surrounding type declarations).

**PASS**: No matches found
**FAIL**: Match found — use epsilon comparison or range check

**Remediation**:
```c
// Before (violation)
if (voltage == 0.0F)

// After (compliant)
if (fabsf(voltage) < 1e-9F)

// Or for zero check:
if (voltage < 1e-9F && voltage > -1e-9F)
```

### Check 6: NULL Pointer Guard

Detect non-static functions with pointer parameters that lack NULL checks near the function entry.

**Tool**: Read + Grep
**Strategy**:
1. Find non-static functions with pointer parameters: `^[a-z].*\*\s*\w+\)` in `.c` files
2. For each match, read the first 10 lines of the function body
3. Check for `if (ptr == NULL)`, `if (!ptr)`, `if (NULL == ptr)`, or assert-style NULL check

**Pattern**: Function signature with `type* param` → body first 10 lines must contain NULL check for each pointer param

**PASS**: All pointer parameters have NULL guard within first 10 lines of function body
**FAIL**: Pointer parameter used without NULL check

**Remediation**:
```c
// Before (violation)
void process_data(uint8_t* buffer, uint32_t size)
{
    buffer[0] = 0;  // potential NULL dereference
}

// After (compliant)
void process_data(uint8_t* buffer, uint32_t size)
{
    if (buffer == NULL)
    {
        return;
    }
    buffer[0] = 0;
}
```

**Scope**: Non-static (public) functions only. Static (file-local) functions are exempt because callers are in the same file and can be verified locally.

### Check 7: Enum Switch Completeness (Rule 8.3+)

Detect `switch` statements on enum-typed variables where not all enum values have explicit `case` labels.

**Tool**: Read + Grep
**Strategy**:
1. Find `switch` blocks: `switch\s*\(` in `.c` files
2. Identify the switched variable's type (trace back to declaration or parameter)
3. If the type is an enum, collect all enum values from the enum definition
4. Compare enum values against `case` labels in the switch block
5. Flag any enum value without a corresponding `case` (even if `default:` exists)

**PASS**: All enum values have explicit `case` labels in the switch
**FAIL**: One or more enum values lack a `case` label and rely solely on `default:`

**Remediation**:
```c
// Before (violation) — STATE_ERROR not handled explicitly
typedef enum { STATE_IDLE, STATE_RUN, STATE_ERROR } state_t;

switch (state)
{
    case STATE_IDLE:
        break;
    case STATE_RUN:
        break;
    default:
        break;
}

// After (compliant) — all values explicit
switch (state)
{
    case STATE_IDLE:
        break;
    case STATE_RUN:
        break;
    case STATE_ERROR:
        handle_error();
        break;
    default:
        break;
}
```

**Note**: `default:` is still required (Check 4), but it should be a safety net, not a substitute for explicit handling.

## Output Format

```markdown
## BARR-C:2018 Verification Report

| # | Rule | Description | Status | Issues |
|---|------|-------------|--------|--------|
| 1 | 1.3 | Braces on control statements | PASS/FAIL | N |
| 2 | 1.7 | No goto/auto/register | PASS/FAIL | N |
| 3 | 8.2b | No assignment in conditions | PASS/FAIL | N |
| 4 | 8.3 | Switch requires default | PASS/FAIL | N |
| 5 | 5.4 | No float equality | PASS/FAIL | N |
| 6 | — | NULL pointer guard | PASS/FAIL | N |
| 7 | 8.3+ | Enum switch completeness | PASS/FAIL | N |

### Issues Found

| # | Rule | File:Line | Code | Remediation |
|---|------|-----------|------|-------------|
```

## Exceptions

The following are **NOT violations**:

1. **CMSIS/vendor headers** — Files under `CMSIS/`, `System/`, or third-party includes
2. **Preprocessor directives** — `#if`, `#ifdef` do not require braces
3. **Comments containing keywords** — `goto`, `auto`, `register` in comments are not violations
4. **String literals** — Keywords inside string literals (`"goto label"`) are not violations
5. **Comparison with integer literal** — `== 0` or `!= 1` with integer context is not a float equality issue
6. **`auto` in C++ context** — Not applicable (this is a C-only project, but noted for completeness)
