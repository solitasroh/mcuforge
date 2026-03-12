---
name: systematic-debugging
description: "Guides systematic 4-step debugging: Investigate (reproduce, evidence), Analyze (narrow scope, trace data flow), Hypothesize (root cause), Fix (minimal change + verify). Use for complex bugs, intermittent issues, or multi-module bugs. Do NOT use for build errors or simple typos."
argument-hint: "<bug description>"
user-invokable: true
agent: systematic-debugger
---

# Systematic Debugging

Apply a rigorous 4-step debugging methodology to find root causes of complex bugs.

## Framework

### Step 1: Investigate (Evidence Collection)
- Clarify reproduction conditions
- Collect relevant logs, state variables, error messages
- Determine timeline: when did this first appear? (`git bisect`)
- Scope: single module or cross-module?

### Step 2: Analyze (Scope Narrowing)
- Trace call chains backward (callee → caller)
- Trace data flow forward (input → transformation → output)
- Check change history (`git log --follow`)
- Eliminate working paths to narrow suspects

### Step 3: Hypothesize (Root Cause Theory)
- Form 1-3 root cause hypotheses
- For each: predict behavior, identify confirming/refuting evidence
- Describe test method

### Step 4: Fix (Minimal Change)
- Propose smallest change testing the hypothesis
- Verify no side effects on other paths
- Identify regression prevention points

## When to Use
- Complex bugs spanning multiple modules
- Intermittent or timing-dependent issues
- Bugs that resist simple inspection
- Issues where the obvious fix didn't work
