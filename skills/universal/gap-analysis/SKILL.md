---
name: gap-analysis
description: "Detects design-vs-implementation gaps: header declarations without implementations, enum values without switch handlers, error codes without return paths, unused struct fields, naming violations. Use before PR review or after significant refactoring. Do NOT use for style checks (use verify skills)."
argument-hint: "[Sources/|Drivers/|all]"
user-invokable: true
agent: gap-detector
---

# Gap Analysis

Detect gaps between design intent and actual implementation in C codebase.

## Detection Categories

| Category | What to Find |
|----------|-------------|
| Declaration-Implementation | Functions in `.h` without `.c` implementation |
| Definition-Usage | Enum values without switch-case, error codes never returned |
| Resource Lifecycle | acquire without release on error paths |
| Naming Convention | Module prefix inconsistency |
| Struct Field Usage | Fields assigned but never read (dead assignment) |
| Init Order | Dependency init called after dependent |

## Workflow

1. Scan all `.h` files for declarations, enums, error codes
2. Cross-reference with `.c` implementations
3. Build include dependency graph
4. Report gaps grouped by severity

## Output Format

```
## Gap Analysis Report

### Critical (require action)
| File | Type | Gap | Details |

### Warning (should address)
| File | Type | Gap | Details |

### Info (improvement opportunity)
| File | Type | Gap | Details |
```
