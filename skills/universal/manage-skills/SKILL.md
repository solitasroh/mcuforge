---
name: manage-skills
description: "Analyzes session changes to detect missing verification skills, creates or updates skills, and syncs CLAUDE.md. Use after adding new coding patterns or rules that need automated verification. Do NOT use for runtime debugging or code generation."
disable-model-invocation: true
argument-hint: "[optional: specific skill name or focus area]"
user-invokable: true
---

# Session-Based Skill Maintenance

## Purpose

Analyzes changes in the current session to detect and correct verification skill drift:

1. **Missing Coverage** — Changed files referenced by no verify skill
2. **Invalid References** — Skills referring to deleted or moved files
3. **Missing Checks** — New patterns/rules not covered by existing checks
4. **Outdated Values** — Configuration values or detection commands that no longer match

## When to Run

- After implementing features introducing new patterns or rules
- When modifying existing verify skills and wanting to check consistency
- Before PR to ensure verify skills cover changed areas
- When a verification run missed an expected issue
- Periodically to align skills with codebase evolution

## Registered Verification Skills

List of verification skills registered in the current project. Updated when creating/deleting skills.

| Skill | Description | Covered File Patterns |
|-------|-------------|-----------------------|
| `verify-mcu-float` | Enforces uppercase F suffix for float literals | `Sources/**/*.c`, `Drivers/**/*.c` |
| `verify-type-safety` | Enforces stdint.h types over primitive types | `Sources/**/*.c`, `Drivers/**/*.c`, `Components/**/*.c` |
| `verify-driver-structure` | Checks driver dependencies and init naming | `Drivers/**/*.c` |
| `verify-barr-c` | Enforces BARR-C:2018 high-priority coding rules | `Sources/**/*.c`, `Drivers/**/*.c` |

## Workflow

### Step 1: Analyze Session Changes

Collect all files changed in the current session:

```bash
# Uncommitted changes
git diff HEAD --name-only

# Commits in current branch (if branched from main/develop)
git log --oneline develop..HEAD 2>/dev/null || git log --oneline main..HEAD 2>/dev/null

# All changes since branching
git diff develop...HEAD --name-only 2>/dev/null || git diff main...HEAD --name-only 2>/dev/null
```

Merge into a deduplicated list. Filter if optional arguments provided.

**Display:** Group files by top-level directory:

```markdown
## Session Change Detection

**N files changed in this session:**

| Directory | Files |
|-----------|-------|
| Sources | `measure.c`, `measure.h` |
| Drivers | `adc.c` |
| (root) | `CMakeLists.txt` |
```

### Step 2: Map Changes to Registered Skills

Build file-to-skill mapping using the **Registered Verification Skills** section above.

#### Sub-step 2a: Check Registered Skills

Read skill names and covered file patterns from the table.

If 0 registered skills: Skip to Step 4. All changed files are "UNCOVERED".

If 1+ registered skills: Read `.claude/skills/verify-<name>/SKILL.md` for additional paths from:
1. **Related Files** section
2. **Workflow** section (paths in grep/glob/read commands)

#### Sub-step 2b: Match Files to Skills

Match each changed file from Step 1 against registered skills. Match if:
- Matches skill's covered file pattern
- Located in directory referenced by skill
- Matches regex/string pattern in skill's detection commands

#### Sub-step 2c: Display Mapping

```markdown
### File → Skill Mapping

| Skill | Trigger File | Action |
|-------|--------------|--------|
| verify-adc | `Sources/measure.c` | CHECK |
| (none) | `CMakeLists.txt` | UNCOVERED |
```

### Step 3: Analyze Coverage Gaps

For each AFFECTED skill, read full SKILL.md and check:

1. **Missing File References** — Are relevant changed files missing from Related Files?
2. **Outdated Detection** — Do grep/glob patterns still match current file structure? Test with sample commands.
3. **Uncovered New Patterns** — Read changed files to identify new rules/patterns not checked.
   - New types, enums, exported symbols
   - New registrations or configurations
   - New naming or directory conventions
4. **Stale References** — Are files in Related Files deleted?
5. **Changed Values** — Are specific values (constants, config keys) checked by skill modified?

Record gaps:

```markdown
| Skill | Gap Type | Details |
|-------|----------|---------|
| verify-adc | Missing File | `Sources/new_adc.c` missing from Related Files |
| verify-config | Outdated Value | Config key `ADC_RATE` rename |
```

### Step 4: Decision - CREATE vs UPDATE

Apply decision tree:

```
For each group of UNCOVERED files:
    IF related to domain of existing skill:
        → DECISION: UPDATE existing skill (expand coverage)
    ELSE IF 3+ related files share common rules/patterns:
        → DECISION: CREATE new verify skill
    ELSE:
        → Mark as "Exempt" (no skill needed)
```

Present results:

```markdown
### Proposed Actions

**DECISION: UPDATE Existing Skills** (N)
- `verify-adc` — Add 2 missing file refs, update patterns

**DECISION: CREATE New Skills** (M)
- New Skill needed — Cover <pattern description> (X uncovered files)

**No Action Needed:**
- `README.md` — Documentation, exempt
```

Confirm with user via `AskUserQuestion`.

### Step 5: Update Existing Skills

For approved updates, edit SKILL.md:

**Rules:**
- **Add/Modify Only** — Do not remove working checks
- Add new paths to **Related Files**
- Add detection commands for new patterns
- Remove references to deleted files
- Update changed values

### Step 6: Create New Skill

**Important:** Must confirm skill name with user.

1. **Explore** — Read related files to understand patterns
2. **Confirm Name** — Ask user for name (must start with `verify-`, kebab-case)
3. **Create** — Generate `.claude/skills/verify-<name>/SKILL.md`:

```yaml
---
name: verify-<name>
description: <One-line description>. Check <Trigger condition>.
---
```

Required Sections:
- **Purpose** — 2-5 categories
- **When to Run** — 3-5 triggers
- **Related Files** — Real file paths (verify with `ls`)
- **Workflow** — Inspection steps (Tool, Path/Pattern, Pass/Fail criteria, Remediation)
- **Output Format** — Markdown table
- **Exceptions** — 2-3 realistic cases

4. **Update Related Files** — Must update 3 files:

   **4a. This file (`manage-skills/SKILL.md`)**:
   - Add row to **Registered Verification Skills** table

   **4b. `verify-implementation/SKILL.md`**:
   - Add row to **Target Skills** table

   **4c. `CLAUDE.md` (or project root doc)**:
   - Add row to **Skills** table

### Step 7: Verification

1. Read all modified SKILL.md files
2. Check Markdown syntax
3. Check for broken file references (`ls <path>`)
4. Dry-run one detection command from updated skills
5. Verify sync between skill tables

### Step 8: Summary Report

```markdown
## Session Skill Maintenance Report

### Analyzed Changed Files: N

### Updated Skills: X
- `verify-<name>`: ...

### Created Skills: Y
- `verify-<name>`: ...

### Updated Related Files:
- `manage-skills/SKILL.md`
- `verify-implementation/SKILL.md`
- `CLAUDE.md`
```

---

## Quality Standards

Created/Updated skills must have:
- **Real file paths** (verified with `ls`)
- **Working detection commands**
- **Clear PASS/FAIL criteria**
- **Realistic Exceptions**
- **Consistent Format**

## Related Files

| File | Purpose |
|------|---------|
| `.claude/skills/verify-implementation/SKILL.md` | Integrated verification skill (manages target list) |
| `.claude/skills/manage-skills/SKILL.md` | This file (manages registered list) |
| `CLAUDE.md` | Project guidelines |

## Exceptions

**NOT Issues:**
1. **Lock/Generated Files**
2. **One-off Config Changes**
3. **Documentation**
4. **Test Fixtures**
5. **Unaffected Skills**
6. **CLAUDE.md Itself**
7. **Vendor/Third-party Code**
8. **CI/CD Config**
