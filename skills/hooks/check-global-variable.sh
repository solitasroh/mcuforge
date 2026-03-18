#!/bin/bash
# [PreToolUse] Write|Edit - warning (additionalContext, no blocking)
# Warn when adding new extern global variables (increases coupling)

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor
hook_require_code

# Detect extern declarations (exclude function declarations with parentheses)
VIOLATIONS=$(echo "$CODE" | grep -nE '^\s*extern\s+' | \
	grep -v '(' | \
	grep -v '^\s*//' || true)

if [ -n "$VIOLATIONS" ]; then
	hook_warn "New extern global variable adds coupling. Prefer static + getter/setter. Found: $(echo "$VIOLATIONS" | head -3)"
fi

exit 0
