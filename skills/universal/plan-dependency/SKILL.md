---
name: plan-dependency
description: "ALWAYS use when analyzing #include dependency graph: circular dependency detection, layer violation checks, fan-in/fan-out coupling metrics, dependency matrix generation, and decoupling strategy recommendations. ALWAYS use when user mentions: 의존성 분석, dependency analysis, include 그래프, include graph, 순환 의존성, circular dependency, 순환 참조, circular reference, 계층 위반, layer violation, 결합도 분석, coupling analysis, 모듈 의존성, module dependency, fan-in, fan-out, 의존성 매트릭스, dependency matrix, 헤더 의존성, header dependency, 모듈 분리, decoupling, 전이적 의존성, transitive dependency, include chain, '#include 패턴 검사', '순환 의존성 있는지 확인', or wants to visualize or analyze header include relationships. Do NOT use for: runtime data flow or variable value tracing through pipeline (use plan-dataflow), function call relationship tracing without #include context (use plan-dataflow), Drivers/ reverse dependency or _init naming checks (use verify-driver-structure), extern usage analysis or cyclomatic complexity metrics (use plan-quality-audit), or comprehensive code quality audit with debt analysis (use plan-quality-audit)."
argument-hint: "[Sources/|Drivers/|all] [--matrix] [--cycles]"
user-invokable: true
agent: dependency-analyzer
---

# Include Dependency Analysis

Analyze `#include` dependency graph across the codebase to detect structural issues.

## When to Use

- Before major refactoring to understand coupling
- When adding new modules to check dependency impact
- When build times increase (deep include chains)
- To enforce architectural layer boundaries

## Workflow

### Step 1: Build Dependency Graph

1. **Scan**: Find all `.c` and `.h` files in scope (Sources/, Drivers/, Components/)
2. **Parse**: Extract `#include` directives from each file
3. **Classify**: Categorize each include as:
   - **System**: `<stdint.h>`, `<string.h>` (ignored in analysis)
   - **Vendor**: CMSIS, MK10D7 headers (noted but not flagged)
   - **Project**: Application, Driver, Component headers (analyzed)
4. **Build Graph**: Create directed graph: file A includes file B → edge A→B

### Step 2: Circular Dependency Detection (`--cycles`)

Find cycles in the include graph using DFS-based cycle detection.

**Output**:
```markdown
### Circular Dependencies

| # | Cycle | Length | Severity |
|---|-------|--------|----------|
| 1 | A.h → B.h → A.h | 2 | Critical |
| 2 | X.h → Y.h → Z.h → X.h | 3 | Critical |

**Remediation**: Forward declarations, interface extraction, or dependency inversion.
```

### Step 3: Layer Violation Detection

Enforce architectural boundaries:

| Layer | Can Include | Cannot Include |
|-------|------------|----------------|
| Sources/ (App) | Drivers/, Components/, CMSIS/ | — |
| Components/ (Middleware) | Drivers/, CMSIS/ | Sources/ |
| Drivers/ (HAL) | CMSIS/ | Sources/, Components/ |
| CMSIS/ (Vendor) | — | Sources/, Components/, Drivers/ |

Flag any include that violates these rules.

### Step 4: Coupling Metrics

For each module (file), calculate:

| Metric | Description | Threshold |
|--------|-------------|-----------|
| **Fan-out** | Number of files this module includes | Warning >8 |
| **Fan-in** | Number of files that include this module | Info only |
| **Afferent coupling (Ca)** | Incoming dependencies | — |
| **Efferent coupling (Ce)** | Outgoing dependencies | — |
| **Instability (I)** | Ce / (Ca + Ce) | 0=stable, 1=unstable |

### Step 5: Dependency Matrix (`--matrix`)

Generate NxN matrix showing module dependencies:

```
          | measure | calib | hv  | dio | index |
----------|---------|-------|-----|-----|-------|
measure   |    —    |   ✓   |  ✓  |     |       |
calib     |         |   —   |  ✓  |     |       |
hv        |         |       |  —  |  ✓  |       |
dio       |         |       |     |  —  |       |
index     |    ✓    |       |  ✓  |  ✓  |   —   |
```

## Output Format

```markdown
## Dependency Analysis Report — <scope>

### Summary
| Metric | Count |
|--------|-------|
| Total files analyzed | N |
| Total include edges | N |
| Circular dependencies | N |
| Layer violations | N |
| Top fan-out file | file.h (N includes) |
| Top fan-in file | file.h (N includers) |

### Top 5 Most Coupled Files
| # | File | Fan-out | Fan-in | Instability |
|---|------|---------|--------|-------------|

### Issues
| # | Type | Files | Description | Remediation |
|---|------|-------|-------------|-------------|
```

## Exceptions

- **CMSIS/vendor headers**: Not flagged for coupling metrics
- **System headers**: `<stdint.h>`, `<stdbool.h>`, etc. excluded from graph
- **Test files**: Files under `tests/` excluded from analysis
