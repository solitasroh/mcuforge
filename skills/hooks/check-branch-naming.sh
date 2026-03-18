#!/bin/bash
# [PreToolUse] Bash - blocking (exit 2)
# Enforce branch naming: feature/OP-XXX-*, fix/OP-XXX-*, refactor/*, develop, main

source "$(dirname "$0")/lib/common.sh"
hook_parse_input

# Only check branch creation commands
echo "$COMMAND" | grep -qE 'git\s+(branch|checkout\s+-b|switch\s+-c)' || exit 0

# Extract branch name
BRANCH_NAME=$(echo "$COMMAND" | grep -oE '(branch|checkout\s+-b|switch\s+-c)\s+([a-zA-Z0-9/_-]+)' | awk '{print $NF}')

[ -z "$BRANCH_NAME" ] && exit 0

# Allowed patterns
echo "$BRANCH_NAME" | grep -qE '^(feature|fix|hotfix)/OP-[0-9]+-' && exit 0
echo "$BRANCH_NAME" | grep -qE '^(refactor|release|chore)/' && exit 0
echo "$BRANCH_NAME" | grep -qE '^(develop|main|master)$' && exit 0

echo "Allowed patterns:" >&2
echo "  feature/OP-XXX-description" >&2
echo "  fix/OP-XXX-description" >&2
echo "  hotfix/OP-XXX-description" >&2
echo "  refactor/description" >&2
echo "Got: $BRANCH_NAME" >&2
hook_block "Branch name does not follow naming convention."
