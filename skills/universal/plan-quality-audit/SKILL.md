---
name: plan-quality-audit
description: "Audits C code structural quality metrics: cyclomatic complexity, function length, nesting depth, parameter count, extern coupling, code duplication, technical debt (TODO/FIXME markers AND implicit debt — commented-out code, #if 0 dead code, active debug flags in release, empty stub functions), logical flaw detection (missing error paths, division-by-zero gaps, float == precision traps, incomplete state machines, resource leaks, race conditions), and refactoring strategies with before/after examples. Focuses on code structure metrics and architectural debt — NOT on embedded hardware safety. ALWAYS use this skill when the user mentions: 코드 품질, 품질 감사, quality audit, 기술 부채, technical debt, 주석 처리된 코드, dead code, 리팩토링, refactoring, 코드 복잡도, 순환 복잡도, cyclomatic complexity, 함수 길이, 중첩 깊이, nesting depth, 결합도, coupling, extern 남용, 코드 스멜, code smell, 논리적 허점, 논리적 결함, logical flaw, 릴리스 전 품질 점검, pre-release quality gate, 스프린트 계획 품질, 코드 감사, or wants multi-file structural analysis. Do NOT use for: ISR safety / volatile / soft-float review (use check-code-review --safety), single-file quick checks (use verify-* hooks), specific bug debugging (use do-debug), binary size analysis (use check-binary-analysis), or unit test generation (use do-test-gen)."
argument-hint: "[Sources/|Drivers/|all] [--debt] [--refactor] [--logic]"
user-invokable: true
agent: code-quality-analyzer
---

# Code Quality Audit

Analyze structural quality metrics across the entire codebase or a specific directory.

## Metrics and Thresholds

See `references/metric-thresholds.md` for detailed rationale behind each threshold.

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
4. **Technical Debt Scan** (always, or `--debt` for dedicated report)
5. **Logical Flaw Scan** (always brief, or `--logic` for dedicated report)
6. **Refactoring Candidates** (always, or `--refactor` for detailed strategies)
7. **Report**: Generate Quality Metrics → Debt Inventory → Logic Flaws → Refactoring Candidates

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

## Technical Debt

Use `--debt` for a dedicated debt inventory report.

Debt detection covers **two layers** — both must always be scanned:

### Layer 1: Explicit Markers

| Marker | Priority | Meaning |
|--------|----------|---------|
| FIXME | High | Known bug or broken behavior |
| HACK | High | Workaround that needs proper fix |
| XXX | Medium | Dangerous or fragile code |
| TODO | Low | Planned improvement |

### Layer 2: Implicit Debt

Explicit markers are often absent in well-maintained codebases. The following implicit debt patterns are equally important and must always be scanned, especially when explicit markers yield 0 results:

| Pattern | Detection Method | Severity |
|---------|-----------------|----------|
| Commented-out code | Grep for `// ` followed by C statements (function calls, assignments, preprocessor) | Medium |
| Active debug flags in release | Grep for `#define.*DEBUG.*1` or `#define.*TRACE.*1` outside `#ifdef DEBUG` guards | **High** |
| Empty/stub functions | Functions with empty body `{ }` or only a comment | Medium |
| Dead code blocks | `if (0)`, `#if 0`, unreachable code after `return` | Low |
| Hardcoded magic numbers | Numeric literals without named constants in business logic (not register addresses) | Low |
| Disabled register map entries | Commented-out entries in configuration tables/arrays | Medium |

### Debt Workflow

1. **Scan explicit**: Grep for TODO/FIXME/HACK/XXX in scope
2. **Scan implicit**: Apply Layer 2 patterns via Grep + Read
3. **Enrich**: For each finding, get age via `git blame`
4. **Classify**: Assign severity (High/Medium/Low) and ID (DEBT-NNN)
5. **Aggregate**: Count by module, type, and severity
6. **Report**: Generate combined inventory

### Debt Output

```
## Technical Debt Report

### Summary
- Explicit markers: N items (TODO: X, FIXME: Y, HACK: Z, XXX: W)
- Implicit debt: N items (commented-code: A, debug-flags: B, dead-code: C, ...)
- Highest severity: <item description>

### By Module
| Module | Explicit | Implicit | Total |
|--------|----------|----------|-------|

### Details (sorted by severity, then age)
| # | ID | File:Line | Type | Severity | Age | Content |
|---|-----|-----------|------|----------|-----|---------|
```

## Logical Flaw Detection

Use `--logic` for a dedicated logical flaw report.

Logical flaws are functional correctness risks that metrics alone cannot reveal. They cause silent failures, data corruption, or undefined behavior under specific conditions.

### Detection Categories

| Category | What to Look For |
|----------|-----------------|
| Missing error paths | Functions that can fail but callers ignore return values; `malloc`/`open` without NULL check |
| Boundary gaps | Division without zero-check; array index without bounds check; `uint` underflow to large positive |
| Incomplete state machines | `switch` without `default`; enum values added but not handled in all switches |
| Float precision traps | `==` comparison on floats; accumulated rounding in loops; `double` → `float` truncation |
| Resource leaks | Acquired resources (mutex, critical section, DMA channel) not released on error paths |
| Race condition windows | Shared variable read-modify-write without atomic/critical section; volatile misuse |
| Silent truncation | Implicit narrowing casts (`uint32_t` → `uint16_t`); shift overflow on signed types |
| Resource acquire/release mismatch | Acquired resource not released on all paths (see Resource Pair Table below) |

### Resource Pair Table

When checking resource leaks, verify that every acquire has a matching release on **all** return paths, including error paths:

| Resource Type | Acquire | Release | Verify |
|--------------|---------|---------|--------|
| Critical Section | `__disable_irq()` / `__get_PRIMASK()` | `__set_PRIMASK()` / `__enable_irq()` | All return paths |
| File I/O | `fopen()` | `fclose()` | Error paths included |
| Memory | `malloc()` / `calloc()` | `free()` | Error paths included |
| DMA | `dma_enable()` | `dma_disable()` | deinit path |
| Mutex | `mutex_lock()` | `mutex_unlock()` | All return paths |
| NVIC | `NVIC_EnableIRQ()` | `NVIC_DisableIRQ()` | deinit path |

### Logic Workflow

1. **Read functions**: Focus on functions with CC >5 or length >40 (higher flaw density)
2. **Trace error paths**: For each function, check what happens when sub-calls fail
3. **Check boundaries**: Identify division, array access, cast operations — verify guards exist
4. **Check state completeness**: For switch/enum, verify all values are handled
5. **Check resource lifecycle**: For acquire/release pairs, verify all paths release
6. **Report**: Each flaw gets severity + concrete fix suggestion

### Logic Output

```
## Logical Flaw Report — <scope>

### Critical (silent failure or data corruption risk)
| # | File:Line | Function | Flaw | Impact | Fix |
|---|-----------|----------|------|--------|-----|

### Warning (edge case risk)
| # | File:Line | Function | Flaw | Impact | Fix |
|---|-----------|----------|------|--------|-----|

### Summary
- Functions analyzed: N
- Flaws found: X critical, Y warning
- Top risk: <most dangerous flaw description>
```

## Refactoring Patterns

Use `--refactor` for detailed refactoring strategy recommendations.

### Supported Patterns

See `references/refactoring-patterns.md` for detailed before/after examples of each pattern.

| Pattern | C Application | Trigger |
|---------|--------------|---------|
| Extract Function | Split long function into logical units | Function >60 lines |
| Replace Conditional with Table | if-else chain → function pointer table | >5 branches |
| Introduce Parameter Object | Multiple params → struct | >5 parameters |
| Consolidate Duplicate Code | Repeated patterns → common function | ≥3 similar blocks |
| Reduce Coupling via Callback | Direct calls → callback separation | Circular deps |

### Refactoring Workflow

1. Read target file(s)
2. Identify refactoring candidates using metrics
3. For each candidate: describe problem, propose pattern, show before/after (use `references/refactoring-patterns.md` as template)
4. Assess risk and dependencies
5. Prioritize by impact and safety

## Data Storage

Save audit results to `.claude/data/quality/` for trend tracking.

**File naming**: `YYYY-MM-DD_<scope>.json`
**When to save**: After Step 7 (Report) completes.

**JSON Schema**:
```json
{
  "date": "2026-03-20",
  "commit": "8438bfe",
  "scope": "Sources",
  "files_analyzed": 6,
  "metrics": {
    "max_cc": { "value": 15, "file": "measure.c", "function": "process_measurement" },
    "max_length": { "value": 82, "file": "insulation_index.c", "function": "evaluate_index" },
    "max_nesting": { "value": 4, "file": "module_management.c", "function": "parse_command" }
  },
  "debt": { "explicit": 3, "implicit": 5, "total": 8 },
  "logic_flaws": { "critical": 1, "warning": 3 },
  "violations": { "warning": 7, "critical": 2 }
}
```
