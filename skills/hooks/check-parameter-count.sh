#!/bin/bash
# [PreToolUse] Write|Edit - warning (additionalContext, no blocking)
# Warn when function has more than 5 parameters

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor
hook_require_code

# Detect function definitions/declarations with 6+ commas (= 6+ params)
VIOLATIONS=$(echo "$CODE" | grep -nE '^[a-zA-Z_].*\(.*,.*,.*,.*,.*,.*\)' | \
	grep -v '^\s*//' | \
	grep -v '#define' || true)

if [ -n "$VIOLATIONS" ]; then
	hook_warn "Function has >5 parameters. Consider packing into a struct. Found: $(echo "$VIOLATIONS" | head -3)"
fi

exit 0
