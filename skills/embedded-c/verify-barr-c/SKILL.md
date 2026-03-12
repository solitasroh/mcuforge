---
name: verify-barr-c
description: "Enforces BARR-C:2018 high-priority coding rules: braces on all if/for/while, no goto/auto/register, no assignment in conditions, switch requires default, no float equality. Use proactively when modifying .c/.h files in Sources/ or Drivers/. Do NOT use for CMSIS vendor headers or System/ startup code."
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

## When to Run

- After modifying `.c` or `.h` files in Sources/ or Drivers/
- Before PR to ensure BARR-C compliance
- When verify-implementation orchestrates verification

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
