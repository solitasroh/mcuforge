#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# Block #if 0 blocks and TODO:remove comments (use git history instead)

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor
hook_require_code

IF_ZERO=$(echo "$CODE" | grep -n '#if\s*0' || true)
TODO_REMOVE=$(echo "$CODE" | grep -niE '//\s*TODO:?\s*remove' || true)

VIOLATIONS="${IF_ZERO}${TODO_REMOVE}"

if [ -n "$VIOLATIONS" ]; then
	echo "Delete dead code instead of commenting out. Git history can recover it." >&2
	echo "Found:" >&2
	echo "$VIOLATIONS" | head -3 >&2
	hook_block "Dead code pattern detected (#if 0 or TODO:remove)."
fi

exit 0
