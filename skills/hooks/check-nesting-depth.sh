#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# Block nesting depth > 3 levels (cognitive complexity)

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor
hook_require_code

# Count max indentation depth via tab characters
# jq -r decodes \t to real tabs, so tab counting works correctly
MAX_DEPTH=0
while IFS= read -r line; do
	# Skip blank lines and comments
	stripped=$(echo "$line" | sed 's/^[[:space:]]*//')
	if [ -z "$stripped" ] || echo "$stripped" | grep -qE '^\s*(//|\*|/\*)'; then
		continue
	fi
	# Count leading tabs
	tabs=$(echo "$line" | sed 's/[^\t]//g' | wc -c)
	tabs=$((tabs - 1))
	if [ "$tabs" -gt "$MAX_DEPTH" ]; then
		MAX_DEPTH=$tabs
	fi
done <<< "$CODE"

# Function body starts at 1 tab, so nesting 3 = 4+ tabs
if [ "$MAX_DEPTH" -ge 5 ]; then
	echo "Deep nesting increases cognitive complexity rapidly." >&2
	echo "Fix: early return, condition inversion, or extract function." >&2
	hook_block "Nesting depth ${MAX_DEPTH} detected (max 3 levels recommended)."
fi

exit 0
