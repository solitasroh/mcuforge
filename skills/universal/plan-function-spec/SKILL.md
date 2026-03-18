---
name: plan-function-spec
description: "ALWAYS use when writing function/module specifications (Design by Contract) or verifying specs for logical errors before implementation. Defines preconditions, postconditions, invariants, error cases, and state transitions, then detects contradictions, missing conditions, and impossible states. ALWAYS use when user mentions: 명세 작성, 함수 명세, function spec, 사전조건, precondition, 사후조건, postcondition, 불변조건, invariant, 계약 설계, contract design, Design by Contract, DbC, 상태 전이 설계, FSM 설계, state transition, 전이 완전성, 인터페이스 계약, API 계약, 에러 조건 정의, 명세 검증, spec verification, 명세 역공학, reverse engineering spec, '사전조건/사후조건 명세 작성해줘', 'FSM 전이 테이블 설계', '논리적 오류 명세로 체크', or wants to formally define function behavior before coding. Do NOT use for: test case list generation or test code writing (use do-test-gen), code quality metrics or cyclomatic complexity analysis (use plan-quality-audit), refactoring strategy or technical debt analysis (use plan-quality-audit), ISR safety or embedded code review (use check-code-review), data flow tracing through pipeline stages (use plan-dataflow), or runtime bug debugging (use do-debug)."
argument-hint: "<function-name|module-name> [--fsm] [--verify]"
user-invokable: true
agent: specification-designer
---

# Function/Module Specification (Design by Contract)

Write specifications before implementation. Verify specs for logical errors before test design.

## TDD Chain Position

```
Requirements -> [plan-function-spec] -> [do-test-gen --design] -> [do-test-gen --stub] -> Implementation -> Verification
                  Spec writing +           Test case list          Failing test code
                  Logic verification
```

## When to Use

- Before implementing new functions or modules
- Before writing tests (spec drives test design)
- When designing state machines (--fsm)
- When reviewing existing code for spec gaps (reverse engineering)

## Arguments

| Argument | Description |
|----------|-------------|
| `<name>` | Function name, module name, or header file path |
| `--fsm` | Include state transition table and FSM completeness analysis |
| `--verify` | Run logical error verification on the specification |

## Workflow

### Step 1: Specification Writing

**Input**: Function signature, header file, or natural language requirements.

#### 1a. Gather Context

1. If function/module exists: Read the header (.h) for signature, Read the source (.c) for current behavior
2. If new: Use user-provided requirements or natural language description
3. Identify related modules and their interfaces

#### 1b. Write Structured Specification

For each function, produce the following specification document:

```markdown
## Function Spec: <function_name>

### Signature
<return_type> <function_name>(<parameters>)

### Purpose
<one-line description of what this function does>

### Preconditions
| # | Condition | Violation Behavior |
|---|-----------|-------------------|
| P1 | <condition on input/state> | <what happens: return error, assert, undefined> |

### Postconditions
| # | Condition | Description |
|---|-----------|-------------|
| Q1 | <condition on output/state after call> | <explanation> |

### Invariants
| # | Condition |
|---|-----------|
| I1 | <condition that must hold before AND after the call> |

### Error Cases
| # | Input Condition | Expected Behavior |
|---|----------------|-------------------|
| E1 | <specific error scenario> | <return value, side effect, or state change> |

### Side Effects
| # | Effect | Description |
|---|--------|-------------|
| S1 | <global state / hardware / IO change> | <details> |
```

#### 1c. State Transition Table (--fsm)

When `--fsm` is specified or the function involves state machine logic:

```markdown
### State Transitions
| Current State | Event | Next State | Guard Condition | Action |
|--------------|-------|------------|-----------------|--------|
| STATE_A | event_x | STATE_B | guard_expr | action_func() |
```

Include:
- All states from the enum definition
- All events/triggers
- Guard conditions for conditional transitions
- Actions performed during transition

### Step 2: Specification Verification (--verify)

Analyze the specification for logical errors. This step catches design bugs before any code is written.

#### 2a. Contradiction Detection

Check for conflicting conditions:
- Precondition allows value X, but postcondition is undefined for X
- Two postconditions that cannot both be true
- Error case overlaps with normal case
- Guard conditions that are mutually exclusive but should coexist

#### 2b. Missing Condition Detection

Check for gaps:
- Input values not covered by any precondition (e.g., NaN, negative, zero, MAX)
- Return paths not covered by postconditions
- Error scenarios without defined behavior
- Concurrency/ISR access without protection conditions
- Resource acquisition without release conditions

#### 2c. Impossible State Detection (--fsm)

For state machines:
- States with no incoming transitions (unreachable)
- States with no outgoing transitions (deadlock)
- Missing event handlers in specific states
- Guard conditions that can never be true

#### 2d. FSM Completeness (--fsm)

| # | Check | Result |
|---|-------|--------|
| F1 | All states have all events handled | PASS/WARN: <missing> |
| F2 | No unreachable states | PASS/WARN: <states> |
| F3 | No deadlock states | PASS/WARN: <states> |
| F4 | Initial state can reach all states | PASS/WARN: <unreachable> |
| F5 | All guard conditions are satisfiable | PASS/WARN: <unsatisfiable> |

#### 2e. Verification Report

```markdown
## Specification Verification Report

### Contradictions
| # | Items | Description |
|---|-------|-------------|
| C1 | P2 vs Q3 | <explanation of conflict> |

### Missing Conditions
| # | Item | Description |
|---|------|-------------|
| M1 | <area> | <what is missing and why it matters> |

### Impossible States (--fsm)
| # | Item | Description |
|---|------|-------------|
| - | None | or <description> |

### Recommendations
1. <specific fix for each issue found>
```

### Step 3: Test Case Derivation

Automatically derive test cases from the specification. This output feeds into `do-test-gen --design`.

For each precondition, postcondition, error case, and boundary:

```markdown
## Derived Test Cases

### From Preconditions
| # | Source | Test Input | Expected Output | Category |
|---|--------|-----------|-----------------|----------|
| T1 | P1 violation | <input violating P1> | <expected error behavior> | Error |
| T2 | P1 boundary | <input at P1 boundary> | <expected behavior> | Boundary |

### From Postconditions
| T3 | Q1 normal | <normal input> | <Q1 condition holds> | Normal |

### From Error Cases
| T4 | E1 | <E1 input> | <E1 expected behavior> | Error |

### From State Transitions (--fsm)
| T5 | S1->S2 | <event in S1> | <transition to S2 + action> | State |
| T6 | S1->S1 | <unhandled event in S1> | <stays in S1 or error> | Negative |
```

This list becomes the input for `do-test-gen --design`.

## Output Format Summary

| Mode | Output |
|------|--------|
| Default | Specification document (Step 1) + Test case list (Step 3) |
| `--verify` | Specification + Verification report (Step 2) + Test case list |
| `--fsm` | Specification with state transitions + FSM completeness check |
| `--fsm --verify` | Full specification + FSM verification + Test cases |

## Reverse Engineering Mode

When applied to existing functions (the function already has an implementation):

1. Read the source code and extract the implicit specification
2. Document what the code actually does (not what it should do)
3. Flag discrepancies between code behavior and reasonable expectations
4. Use `--verify` to check the extracted specification for logical gaps

This is useful for:
- Understanding legacy code before refactoring
- Finding hidden assumptions in existing implementations
- Generating regression test cases from actual behavior

## Exceptions

- **Trivial getters/setters**: Functions that only read/write a single variable do not need full specification
- **ISR handlers**: Use simplified spec (trigger condition, timing constraint, shared state list)
- **Vendor/CMSIS code**: Do not specify third-party code
