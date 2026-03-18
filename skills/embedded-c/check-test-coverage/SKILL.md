---
name: check-test-coverage
description: "ALWAYS use when measuring code coverage after test execution: gcov/lcov collection, per-module function/line/branch coverage reporting, uncovered function identification, and coverage gate threshold checks. ALWAYS use when user mentions: 테스트 커버리지, test coverage, 커버리지 측정, coverage measurement, gcov, lcov, 커버리지 리포트, coverage report, 라인 커버리지, line coverage, 브랜치 커버리지, branch coverage, 함수 커버리지, function coverage, 미커버 함수, uncovered functions, 커버리지 게이트, coverage gate, 커버리지 퍼센트, coverage percentage, 커버리지 확인, coverage check, '커버리지 몇 퍼센트인지', '미커버 함수 목록 보여줘', '80% 미달 모듈 알려줘', or wants to know which code paths are not exercised by tests. Do NOT use for: writing or generating test code (use do-test-gen), test strategy or test case design (use do-test-gen --design), building test executables or cmake --preset Test (use do-build), test infrastructure setup like fff fakes or stub headers (use do-test-gen), Flash/RAM binary size analysis (use check-binary-analysis), or running ctest without coverage analysis intent."
argument-hint: "[module] [--threshold=N] [--branch]"
user-invokable: true
---

# Test Coverage Analysis

Run unit tests and measure code coverage to identify untested code paths.

## Prerequisites

- `tests/` directory must exist (created by `do-test-gen`)
- Tests must be buildable via `cmake --preset Test && cmake --build --preset Test`
- Host toolchain must support gcov (GCC) or equivalent

## Workflow

### Step 1: Build Tests with Coverage

```bash
cd tests
cmake --preset Test -DCMAKE_C_FLAGS="--coverage -fprofile-arcs -ftest-coverage"
cmake --build --preset Test
```

If CMake preset does not exist, configure manually:
```bash
cmake -B build-coverage -DCMAKE_C_FLAGS="--coverage" -DCMAKE_BUILD_TYPE=Debug
cmake --build build-coverage
```

### Step 2: Run Tests

```bash
ctest --preset Test --output-on-failure
```

Or run test executables directly if ctest preset is not configured.

Record test results: passed, failed, total.

### Step 3: Collect Coverage Data

```bash
# Generate .gcov files
gcov -o build-coverage/ Sources/*.c

# Or use lcov for HTML report
lcov --capture --directory build-coverage --output-file coverage.info
lcov --remove coverage.info '/usr/*' 'tests/*' 'CMSIS/*' --output-file coverage_filtered.info
```

### Step 4: Analyze Coverage

For each source module under test:

| Metric | Description |
|--------|-------------|
| **Function coverage** | % of functions executed at least once |
| **Line coverage** | % of executable lines executed |
| **Branch coverage** | % of branch conditions evaluated both true and false |

### Step 5: Generate Report

```markdown
## Test Coverage Report

### Test Results
- Total: N tests
- Passed: N
- Failed: N

### Coverage Summary
| Module | Functions | Lines | Branches |
|--------|-----------|-------|----------|
| calibration.c | 80% (4/5) | 72% | 60% |
| measure.c | 60% (3/5) | 55% | 45% |
| **Total** | **70%** | **63%** | **52%** |

### Uncovered Functions
| # | Module | Function | Lines | Reason |
|---|--------|----------|-------|--------|
| 1 | measure.c | apply_temp_comp() | 45-67 | No test case |
| 2 | calibration.c | calibrate_current() | 89-102 | Untested error path |

### Coverage Gate
| Metric | Threshold | Actual | Status |
|--------|-----------|--------|--------|
| Function | 80% | 70% | FAIL |
| Line | 70% | 63% | FAIL |
| Branch | 50% | 52% | PASS |
```

## Arguments

| Argument | Description |
|----------|-------------|
| `[module]` | Specific module to analyze (e.g., `calibration.c`). Default: all tested modules |
| `--threshold=N` | Coverage gate threshold percentage. Default: 80% function, 70% line, 50% branch |
| `--branch` | Include branch coverage analysis (requires gcov -b) |

## Coverage Gate Thresholds

Default thresholds (configurable via `--threshold`):

| Metric | Default | Recommended for Safety |
|--------|---------|----------------------|
| Function coverage | 80% | 100% |
| Line coverage | 70% | 90% |
| Branch coverage | 50% | 80% |

## Integration with TDD Workflow

```
[Do] do-test-gen --design → do-test-gen --stub → implement → do-build
  ↓
[Check] check-test-coverage → (if below threshold) → do-test-gen (add cases) → re-run
```

Coverage results help identify:
1. Functions that need test cases → feed back to `do-test-gen`
2. Error paths not exercised → add error test cases
3. Branches not covered → add boundary test cases

## Exceptions

- **Vendor/CMSIS code**: Excluded from coverage analysis
- **System startup code**: `System/` files excluded
- **Test infrastructure**: `tests/` directory excluded from coverage targets
- **Hardware-dependent code**: Functions with heavy HW dependencies (hv.c ISRs) may have lower expected coverage
