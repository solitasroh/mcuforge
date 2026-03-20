# Metric Thresholds — Rationale

## Cyclomatic Complexity (CC)

| Threshold | Severity | Rationale |
|-----------|----------|-----------|
| ≤10 | OK | McCabe's original recommendation. Testable with manageable number of paths. |
| 11-20 | Warning | Requires careful testing. Consider extract-function refactoring. |
| >20 | Critical | Too many paths for reliable testing. Split immediately. |

**Embedded context**: Functions like `parse_command()` or state machine handlers may naturally have higher CC. If CC >10 but each case is simple (e.g., switch with independent cases), this is acceptable with a comment explaining the structure.

## Function Length

| Threshold | Severity | Rationale |
|-----------|----------|-----------|
| ≤60 lines | OK | Fits on one screen. Easy to reason about. |
| 61-100 | Warning | Starting to lose readability. Look for natural split points. |
| >100 | Critical | Too long for safe review. Extract logical sections. |

**Embedded context**: Init functions that configure multiple peripherals sequentially may exceed 60 lines but are still acceptable if each step is a clear block.

## Nesting Depth

| Threshold | Severity | Rationale |
|-----------|----------|-----------|
| ≤3 | OK | Easy to follow control flow. |
| 4 | Warning | Consider early return or guard clauses. |
| >5 | Critical | Nearly impossible to reason about all paths. |

**Embedded context**: ISR handlers should have nesting ≤2. Deep nesting in ISR = blocking risk.

## Parameter Count

| Threshold | Severity | Rationale |
|-----------|----------|-----------|
| ≤5 | OK | Manageable cognitive load. |
| >5 | Warning | Consider introducing a parameter struct. |

**Embedded context**: Calibration functions may need many parameters (gain, offset, min, max). Use `calibration_params_t` struct pattern instead.

## Module Coupling (extern)

| Threshold | Severity | Rationale |
|-----------|----------|-----------|
| ≤5 extern/file | OK | Reasonable cross-module dependency. |
| >5 | Warning | High coupling. Consider callback or interface pattern. |

**Embedded context**: `module_management.c` legitimately accesses many modules for register map. This is an exception, not a violation.
