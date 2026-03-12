---
name: mcu-safety-review
description: "Verifies ISR safety, volatile usage, critical sections, and stack safety in firmware code. Use proactively when modifying ISR handlers, shared variables, or interrupt-related code in Sources/ or Drivers/. Do NOT use for pure data structure code or documentation."
argument-hint: "[file or directory]"
user-invokable: true
---

# MCU Safety Review

Automatically verify ISR safety, volatile usage, critical sections, and stack safety.

## Verification Items

| # | Check | Pattern | Target |
|---|-------|---------|--------|
| 1 | ISR blocking calls | ISR function containing `while`, `delay`, `malloc`, `printf` | Sources/, Drivers/ |
| 2 | Missing volatile | Variables shared between ISR and main without `volatile` | Sources/, Drivers/ |
| 3 | Non-atomic 64-bit access | `uint64_t`, `double` shared between ISR and main | Sources/ |
| 4 | Missing critical section | Read-modify-write on shared variables without protection | Sources/ |
| 5 | Large stack allocation | Local variables >128 bytes in a function | Sources/, Drivers/ |

## Workflow

1. **Find ISR handlers**: Grep for `*_IRQHandler` functions
2. **Analyze ISR body**: Check for blocking calls, non-reentrant functions
3. **Find shared variables**: Variables accessed in both ISR and non-ISR context
4. **Check volatile**: Verify shared variables have `volatile` qualifier
5. **Check critical sections**: Verify read-modify-write on shared data is protected
6. **Stack check**: Calculate local variable sizes per function

## Output Format

```
## MCU Safety Report

### Critical (must fix)
| File:Line | Issue | Details |

### Warning (should fix)
| File:Line | Issue | Details |

### Summary
- ISR handlers checked: N
- Shared variables found: N
- Issues: N critical, N warning
```
