---
name: code-quality-audit
description: "Audits C code quality across codebase: function complexity, length, nesting depth, parameter count, global variable usage, module coupling metrics. Use when preparing for refactoring, sprint planning, or debt review. Do NOT use for single-file quick checks (use hooks instead)."
argument-hint: "[Sources/|Drivers/|all]"
user-invokable: true
agent: code-quality-analyzer
---

# Code Quality Audit

Analyze structural quality metrics across the entire codebase or a specific directory.

## Metrics and Thresholds

| Metric | Threshold | Severity when exceeded |
|--------|-----------|----------------------|
| Cyclomatic Complexity | ≤10 per function | Warning >10, Critical >20 |
| Function Length | ≤60 lines | Warning >60, Critical >100 |
| Nesting Depth | ≤3 levels | Warning >3, Critical >5 |
| Parameter Count | ≤5 per function | Warning >5 |
| Module Coupling | Minimize extern | Warning >5 extern per file |
| Code Duplication | <3 similar patterns | Warning ≥3 |

## Workflow

1. **Scope Selection**: Use argument or default to `Sources/`
2. **Per-File Analysis**: Read each `.c` file, calculate all metrics per function
3. **Aggregate**: Summarize by file and directory
4. **Report**: Generate markdown table with violations highlighted
5. **Recommend**: List top 5 refactoring candidates with suggested actions

## Output Format

```
## Code Quality Report — <scope>

### Summary
| Metric | Files | Violations | Pass Rate |
|--------|-------|------------|-----------|

### Per-File Details
| File | Max Func Length | Max Nesting | Extern Count | Complex Functions |
|------|----------------|-------------|--------------|-------------------|

### Top 5 Refactoring Candidates
1. file.c:function() — reason, suggested action
```
