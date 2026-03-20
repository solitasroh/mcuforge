---
name: act-commit
description: "ALWAYS use when committing code changes with Conventional Commits format. ALWAYS use to analyze staged changes, run check-verify, extract OP issue ID from branch, and generate structured commit messages. Do NOT use for amending published commits or force-pushing, changelog generation (use act-changelog), or MR creation (use act-gitlab-mr)."
argument-hint: "[--size-impact] [--amend]"
user-invokable: true
---

# Commit

Analyze staged changes, verify implementation rules, generate a structured commit message following team conventions (including Issue ID), and optionally include firmware binary size impact.

## Arguments

- `--size-impact`: Include Flash/RAM size delta in commit message by comparing before/after build
- `--amend`: Amend the previous commit instead of creating a new one

## Behavior

1. **Verify Implementation** (Critical Step)
   - Run `/check-verify` to check for style violations, missing docs, or known bad patterns (e.g., missing `0.0F` suffix).
   - If issues are found, strictly warn the user and ask whether to abort or proceed.

2. **Identify project & context**
   - Read `CLAUDE.md` for project name and build configuration.
   - **Determine Issue ID (OP#)**:
     - Check current branch name for pattern like `feature/OP-123-...`.
     - If found, extract `OP#123`.
     - If not found, ask user: "Enter Issue ID (e.g., OP#123) or leave empty."

3. **Check staged changes**
   - Run `git diff --cached --stat` and `git diff --cached` to understand what is being committed.
   - If nothing is staged, show unstaged changes (`git status`) and ask what to stage (e.g., `git add .` or specific files).

4. **Analyze changes**
   - Read the staged diffs and categorize:
     - Which components/modules are affected
     - Type of change: `feat`, `fix`, `refactor`, `docs`, `build`, `test`
     - Files modified, added, or deleted

5. **Generate commit message** in Team Convention format:

   ```
   [<IssueID>] <type>(<scope>): <short description>

   <body - what changed and why>

   Components: <list of affected components>
   [Size impact: Flash +1.2KB, RAM +128B (<project_name> Debug)]
   ```

   **Example**: `[OP#123] feat(uart): add parity error detection`

   **Type rules**:
   - `feat`: New functionality
   - `fix`: Bug fix
   - `refactor`: Code restructuring without behavior change
   - `perf`: Performance improvement
   - `docs`: Documentation only
   - `build`: Build system or toolchain changes
   - `chore`: Maintenance tasks

   **Scope**: Component name or module name (e.g., `uart`, `flash`, `adc`)

6. **Size impact** (when `--size-impact` is specified):
   - Run `arm-none-eabi-size` on existing ELF (from CLAUDE.md "Build Output") before
   - Build the project using CLAUDE.md "Build Commands"
   - Run `arm-none-eabi-size` on new ELF after
   - Calculate delta and include in commit message footer

7. **Present to user**
   - Show the generated commit message.
   - Ask for confirmation before committing. Allow edits to the message.

8. **Commit**
   - Execute `git commit` with the approved message.

## Commit Message Examples

```
[OP#123] feat(uart): add parity error detection and recovery

Implement parity error detection in UART RX ISR with automatic
recovery by flushing the RX buffer and re-enabling the receiver.

Components: uart, itask
Size impact: Flash +896B, RAM +0B (Debug)
```

```
[OP#124] fix(flash): prevent write to protected sector

Add sector bounds check before flash erase/program operations.
Previously, writing to sector 0 (bootloader area) would corrupt
the bootloader.

Components: flash
```
