#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# .c/.h float literals must have F suffix (Cortex-M4 soft-float, no double)

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor
hook_require_code

# Detect float literals without F/f suffix
# Pattern 1: decimal (0.0, 3.14, 1.5e3)
# Pattern 2: scientific without decimal (1e-9, 2E3)
# Exclude: comments, #include, #define VERSION, string literals, already-suffixed
DECIMAL_VIOLATIONS=$(echo "$CODE" | grep -nE '[^a-zA-Z_]([0-9]+\.[0-9]+([eE][+-]?[0-9]+)?)[^FfLUlu0-9.]' | \
	grep -v '^\s*//' | \
	grep -v '^\s*\*' | \
	grep -v '#include' | \
	grep -v '#define.*VERSION' | \
	grep -v '@version' | \
	grep -v '@since' | \
	grep -vE '\.[0-9]+F' | \
	grep -vE '"[^"]*[0-9]+\.[0-9]+[^"]*"' || true)

# Pattern 2: bare scientific notation without decimal point (1e-9, 2E3) missing F
SCIENTIFIC_VIOLATIONS=$(echo "$CODE" | grep -nE '\b[0-9]+[eE][+-]?[0-9]+\b' | \
	grep -vE '[0-9]+[eE][+-]?[0-9]+[Ff]' | \
	grep -v '^\s*//' | \
	grep -v '^\s*\*' | \
	grep -v '#include' || true)

VIOLATIONS="${DECIMAL_VIOLATIONS}${SCIENTIFIC_VIOLATIONS}"

if [ -n "$VIOLATIONS" ]; then
	echo "Cortex-M4 soft-float: double ops cause performance penalty." >&2
	echo "Fix: 0.0 -> 0.0F, 1e-9 -> 1e-9F, 3.14 -> 3.14F" >&2
	echo "Violations:" >&2
	echo "$VIOLATIONS" | head -5 >&2
	hook_block "float literal missing F suffix."
fi

exit 0
