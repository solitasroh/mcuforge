---
name: todo-debt-tracker
description: "Scans codebase for TODO, FIXME, HACK, XXX comments and generates a technical debt inventory: count by module, age (git blame), priority estimation, trend. Use during sprint planning, before releases, or debt review. Do NOT use for runtime bug tracking or feature planning."
user-invokable: true
agent: report-generator
---

# Technical Debt Tracker

Scan codebase for technical debt markers and generate an inventory report.

## Markers

| Marker | Priority | Meaning |
|--------|----------|---------|
| FIXME | High | Known bug or broken behavior |
| HACK | High | Workaround that needs proper fix |
| XXX | Medium | Dangerous or fragile code |
| TODO | Low | Planned improvement |

## Workflow

1. **Scan**: Grep for TODO/FIXME/HACK/XXX in Sources/, Drivers/, Components/
2. **Enrich**: For each marker, get age via `git blame`
3. **Aggregate**: Count by module and marker type
4. **Report**: Generate markdown table with severity and age

## Output Format

```
## Technical Debt Report

### Summary
- Total: N items (TODO: X, FIXME: Y, HACK: Z, XXX: W)
- Oldest: file:line (N days ago)

### By Module
| Module | TODO | FIXME | HACK | Total |
|--------|------|-------|------|-------|

### Details (sorted by priority)
| # | File:Line | Type | Age | Content |
|---|-----------|------|-----|---------|
```
