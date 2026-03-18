#!/bin/bash
# [PreToolUse] Write|Edit - warning (additionalContext, no blocking)
# Warn when written code block exceeds 100 lines and contains function definitions

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath

case "$FILE_PATH" in
	*.c) ;;
	*) exit 0 ;;
esac

hook_skip_vendor
hook_require_code

LINE_COUNT=$(echo "$CODE" | wc -l)

# Pass if 100 lines or fewer
if [ "$LINE_COUNT" -le 100 ]; then
	exit 0
fi

# Check if code contains function definitions
HAS_FUNC=$(echo "$CODE" | grep -cE '^[a-zA-Z_][a-zA-Z0-9_*\s]+\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\(' || true)

if [ "$HAS_FUNC" -gt 0 ]; then
	hook_warn "Code block is ${LINE_COUNT} lines. Keep functions under 60 lines (SRP). Consider Extract Function refactoring."
fi

exit 0
