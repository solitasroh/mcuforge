#!/bin/bash
# [PreToolUse] Bash - blocking (exit 2)
# Block dangerous shell commands: force push, hard reset, recursive rm /

source "$(dirname "$0")/lib/common.sh"
hook_parse_input

[ -z "$COMMAND" ] && exit 0

# Dangerous command patterns
DANGEROUS_PATTERNS=(
	'git\s+push\s+.*(-f|--force)'
	'git\s+reset\s+--hard'
	'rm\s+-rf\s+/'
	'rm\s+-rf\s+\.'
	':\(\)\s*\{\s*:\|:&\s*\};:'
)

for pattern in "${DANGEROUS_PATTERNS[@]}"; do
	if echo "$COMMAND" | grep -qE "$pattern"; then
		echo "Detected: $COMMAND" >&2
		hook_block "Dangerous command blocked. Use safer alternatives or confirm manually."
	fi
done

exit 0
