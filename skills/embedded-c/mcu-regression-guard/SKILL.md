---
name: mcu-regression-guard
description: "Compares firmware binary size, stack depth, and memory usage between current branch and base branch to detect regressions. Use before PR or after significant changes. Do NOT use for feature comparison or functional testing."
argument-hint: "[--base=develop]"
user-invokable: true
---

# MCU Regression Guard

Compare firmware metrics between branches to detect regressions.

## Comparison Items

| Item | Method | Warning Threshold |
|------|--------|------------------|
| .text (Flash) | arm-none-eabi-size | >1KB increase |
| .data (RAM init) | arm-none-eabi-size | >256B increase |
| .bss (RAM zero) | arm-none-eabi-size | >512B increase |
| Stack (worst case) | .su files or estimation | >128B increase |

## Workflow

1. **Build current branch**: `cmake --build --preset Release`
2. **Record current metrics**: `arm-none-eabi-size output/*.elf`
3. **Build base branch** (in worktree): Same build commands
4. **Record base metrics**
5. **Compare**: Calculate deltas for each metric
6. **Report**: Flag regressions exceeding thresholds

## Output Format

```
## Regression Report — current vs <base>

| Item | Base | Current | Delta | Status |
|------|------|---------|-------|--------|
| .text (Flash) | 45,280 | 45,512 | +232 | OK |
| .data (RAM) | 1,024 | 1,024 | 0 | OK |
| .bss (RAM) | 8,192 | 8,704 | +512 | WARN |
| Stack (worst) | 892 | 1,100 | +208 | WARN |

### Verdict: PASS | WARN | FAIL
```
