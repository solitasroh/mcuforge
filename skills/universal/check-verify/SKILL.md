---
name: check-verify
description: "ALWAYS use when running all verify-* skills sequentially to generate a consolidated verification report. ALWAYS use after implementation for compliance check, before PR, or during code review for rule enforcement. Do NOT use for single-rule checks (use individual verify-* skills), binary/memory analysis (use check-binary-analysis), or embedded safety review (use check-code-review)."
disable-model-invocation: true
argument-hint: "[optional: specific verify skill name]"
user-invokable: true
---

# Implementation Verification

## Purpose

Performs integrated verification by sequentially executing all `verify-*` skills registered in the project:

- Executes checks defined in each skill's Workflow
- References each skill's Exceptions to prevent false positives
- Provides remediation suggestions for identified issues
- Re-verifies after user approval and fix application

## When to Run

- After implementing new features
- Before creating a Pull Request
- During code review
- When auditing codebase rule compliance

## Target Skills

This list constitutes the verification skills executed sequentially by this skill. 이 목록은 수동으로 관리합니다.

| # | Skill | Description |
|---|-------|-------------|
| 1 | `verify-float-suffix` | Check uppercase 'F' suffix for float literals |
| 2 | `verify-type-safety` | Enforce explicit-width integers from stdint.h |
| 3 | `verify-driver-structure` | Ensure hardware drivers follow architectural boundaries |
| 4 | `verify-barr-c` | Enforce BARR-C:2018 high-priority coding rules |
| 5 | `verify-safety-c` | MISRA-C/CERT-C core safety: signed overflow, buffer bounds, uninitialized vars |

## Workflow

### Step 1: Introduction

Check the **Target Skills** section above.

If optional arguments are provided, filter for those specific skills.

**If 0 skills are registered:**

```markdown
## Implementation Verification

No verification skills found. 수동으로 verify 스킬을 생성하세요.
```

Terminate the workflow.

**If 1 or more skills are registered:**

Display the Target Skills table:

```markdown
## Implementation Verification

Sequentially executing the following verification skills:

| # | Skill | Description |
|---|-------|-------------|
| 1 | verify-<name1> | <description1> |
| 2 | verify-<name2> | <description2> |

Starting verification...
```

### Step 2: Sequential Execution

For each skill listed in the **Target Skills** table:

#### 2a. Read SKILL.md

Read the skill's `.claude/skills/verify-<name>/SKILL.md` and parse the following sections:

- **Workflow** — Inspection steps and detection commands to execute
- **Exceptions** — Patterns considered non-violations
- **Related Files** — List of files to inspect

#### 2b. Execute Inspection

Sequentially execute each check defined in the Workflow section:

1. Detect patterns using the specified tool (Grep, Glob, Read, Bash)
2. Match detected results against the skill's PASS/FAIL criteria
3. Exempt patterns matching the Exceptions section
4. Record issues if FAIL:
   - File path and line number
   - Issue description
   - Remediation recommendation (with code example)

#### 2c. Log Skill Result

Display progress after each skill completes:

```markdown
### verify-<name> Complete

- Checks: N
- Passed: X
- Issues: Y
- Exempt: Z

[Proceeding to next skill...]
```

### Step 3: Consolidated Report

After all skills complete, consolidate results into a single report:

```markdown
## Implementation Verification Report

### Summary

| Verify Skill | Status | Issues | Details |
|--------------|--------|--------|---------|
| verify-<name1> | PASS / X Issues | N | Details... |
| verify-<name2> | PASS / X Issues | N | Details... |

**Total Issues Found: X**
```

**If all pass:**

```markdown
All verifications passed!

Implementation complies with all project rules:

- verify-<name1>: <summary of pass>
- verify-<name2>: <summary of pass>

Ready for code review.
```

**If issues found:**

List each issue with file path, problem description, and remediation recommendation:

```markdown
### Found Issues

| # | Skill | File | Problem | Remediation |
|---|-------|------|---------|-------------|
| 1 | verify-<name1> | `path/to/file.ts:42` | Description | Code example |
| 2 | verify-<name2> | `path/to/file.tsx:15` | Description | Code example |
```

### Step 4: User Action Check

If issues are found, confirm with user using `AskUserQuestion`:

```markdown
---

### Remediation Options

**X issues found. How would you like to proceed?**

1. **Fix All** - Automatically apply all recommended fixes
2. **Fix Individually** - Review and apply fixes one by one
3. **Skip** - Exit without changes
```

### Step 5: Apply Fixes

Apply fixes based on user selection.

**If "Fix All" selected:**

Apply all fixes sequentially and show progress:

```markdown
## Applying Fixes...

- [1/X] verify-<name1>: `path/to/file.ts` fixed
- [2/X] verify-<name2>: `path/to/file.tsx` fixed

X fixes applied.
```

**If "Fix Individually" selected:**

Show fix content for each issue and confirm approval via `AskUserQuestion`.

### Step 6: Re-verification

If fixes were applied, re-execute only the skills that had issues to compare Before/After:

```markdown
## Re-verification

Re-executing skills with issues...

| Verify Skill | Before | After |
|--------------|--------|-------|
| verify-<name1> | X Issues | PASS |
| verify-<name2> | Y Issues | PASS |

All verifications passed!
```

**If issues remain:**

```markdown
### Remaining Issues

| # | Skill | File | Problem |
|---|-------|------|---------|
| 1 | verify-<name> | `path/to/file.ts:42` | Auto-fix unavailable — Manual check required |

Please resolve manually and re-run `/check-verify`.
```

---

## Exceptions

The following are **NOT issues**:

1. **No Registered Skills** — Display guidance message instead of error
2. **Skill's Own Exceptions** — Patterns defined in Exceptions section of each verify skill are not reported
3. **check-verify Itself** — Must not include itself in target list

## Related Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Project guidelines |
