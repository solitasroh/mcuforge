---
name: change-impact-analysis
description: "Analyzes impact of code changes: affected modules, calling functions, data flows, required tests, documentation updates. Use before committing significant changes or when reviewing PRs. Do NOT use for trivial single-line fixes."
argument-hint: "[file or function]"
user-invokable: true
agent: report-generator
---

# Change Impact Analysis

Analyze the ripple effects of code changes across the codebase.

## Analysis Steps

1. **Identify Changes**: Read git diff or specified file changes
2. **Caller Analysis**: Find all functions that call modified functions (Grep)
3. **Data Flow**: Trace data that passes through modified code
4. **Module Impact**: List all modules that depend on changed interfaces
5. **Test Impact**: Identify which tests should be run
6. **Documentation**: Flag any docs that reference changed APIs

## Output Format

```
## Change Impact Report — <scope>

### Modified Functions
| Function | File | Callers | Risk |
|----------|------|---------|------|

### Affected Modules
| Module | Dependency Type | Impact |
|--------|----------------|--------|

### Required Actions
- [ ] Run tests: ...
- [ ] Update docs: ...
- [ ] Verify callers: ...
```
