---
name: refactoring-guide
description: "Suggests refactoring strategies for C code based on code smells: Extract Function, Replace Conditional with Table, Introduce Parameter Object, Consolidate Duplicates, Reduce Coupling. Use when code-quality-audit reports issues or before major changes. Do NOT use for trivial renaming or new feature design."
argument-hint: "<file or function name>"
user-invokable: true
agent: refactoring-advisor
---

# Refactoring Guide

Analyze code and suggest safe refactoring strategies with before/after examples.

## Supported Patterns

| Pattern | C Application | Trigger |
|---------|--------------|---------|
| Extract Function | Split long function into logical units | Function >60 lines |
| Replace Conditional with Table | if-else chain → function pointer table | >5 branches |
| Introduce Parameter Object | Multiple params → struct | >5 parameters |
| Consolidate Duplicate Code | Repeated patterns → common function | ≥3 similar blocks |
| Reduce Coupling via Callback | Direct calls → callback separation | Circular deps |

## Workflow

1. Read target file(s)
2. Identify refactoring candidates using metrics
3. For each candidate: describe problem, propose pattern, show before/after
4. Assess risk and dependencies
5. Prioritize by impact and safety
