---
name: pdca-cycle
description: "Executes Plan-Do-Check-Act improvement cycles for complex problems: Plan (hypothesis) → Do (implement) → Check (verify) → Act (document). Use for recurring bugs, performance optimization, or problems requiring iterative refinement. Do NOT use for simple one-shot tasks."
argument-hint: "<problem description>"
user-invokable: true
agent: pdca-iterator
---

# PDCA Cycle

Execute Plan-Do-Check-Act improvement cycles for complex problem solving.

## Cycle Structure

### Plan
1. Define problem clearly (symptoms, scope, impact)
2. Analyze root cause candidates (5-Why or fishbone)
3. Form hypothesis: "If we change X, then Y because Z"
4. Define measurable success criteria
5. Identify risks and rollback plan

### Do
1. Implement minimum change to test hypothesis
2. Keep changes isolated and reversible
3. Document exactly what was changed and why

### Check
1. Verify against success criteria
2. Run tests / build / measure
3. Compare before vs after
4. Check for side effects

### Act
1. If successful: document fix and root cause
2. If partial: refine hypothesis, next cycle
3. If failed: revert, update understanding
4. Document learnings

## Rules
- Maximum 3 cycles per session
- Each cycle must show measurable progress
- If no progress after 2 cycles, escalate to broader investigation
