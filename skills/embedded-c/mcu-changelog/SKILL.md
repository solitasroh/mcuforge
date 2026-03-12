---
name: mcu-changelog
description: "Generates a structured changelog from git commit history, grouped by component and change type (feat/fix/refactor), with optional binary size delta. Use before release or for release notes. Do NOT use for single-commit summaries."
argument-hint: "[--from=<tag|commit>] [--to=<tag|commit>]"
user-invokable: true
---

# MCU Changelog

Generate a structured changelog from git commit history, grouped by component and change type.

## Arguments

- `--from=<tag|commit>`: Start point (default: latest tag, or first commit if no tags)
- `--to=<tag|commit>`: End point (default: HEAD)

## Behavior

1. **Identify project**: Read `CLAUDE.md` for project name and build output path.

2. **Determine range**:
   - Find the latest git tag with `git describe --tags --abbrev=0`
   - Use HEAD as end point unless `--to` is specified

3. **Collect commits**: Run `git log --oneline --no-merges <from>..<to>`.
   - Filter to commits touching project files: `Sources/`, `System/`, `Drivers/`, `Components/`

4. **Parse and categorize**: For each commit:
   - Parse Conventional Commits format if present
   - If not conventional format, analyze diff to determine type and components
   - Get list of changed files for component scope

5. **Generate changelog**:

   ```markdown
   # Changelog

   ## v1.2.0 (2026-02-08)

   **Range**: abc1234...def5678 (15 commits)

   ### Features
   - **uart**: Add parity error detection and recovery (abc1234)

   ### Bug Fixes
   - **flash**: Prevent write to protected bootloader sector (cde3456)

   ### Components Changed
   | Component | Commits | Files Changed |
   |-----------|---------|---------------|
   | uart      | 3       | 5             |

   ### Binary Size Impact
   | Metric | Before | After | Delta |
   |--------|--------|-------|-------|
   | Flash  | 186.2 KB | 188.4 KB | +2.2 KB |
   ```

## Commit Classification Rules

| Prefix | Category |
| ------ | -------- |
| `feat` | Features |
| `fix` | Bug Fixes |
| `refactor` | Refactoring |
| `perf` | Performance |
| `docs` | Documentation |
| `build` | Build System |
| `test` | Tests |
| `chore` | Maintenance |
| (none) | Uncategorized — classify by analyzing the diff |
