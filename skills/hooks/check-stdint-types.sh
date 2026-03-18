#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# Enforce stdint.h types (uint8_t, int32_t) instead of bare int, unsigned char, etc.

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor
hook_require_code

# Detect bare primitive types in variable/parameter declarations
VIOLATIONS=$(echo "$CODE" | grep -nE '\b(unsigned\s+)?(short|long)\b|\bunsigned\s+char\b|\bsigned\s+(int|char)\b' || true)

# Detect standalone 'int' (allow int main, int argc, intN_t, uintN_t)
INT_VIOLATIONS=$(echo "$CODE" | grep -nE '\bint\b' | \
	grep -vE '\bint(8|16|32|64)_t\b' | \
	grep -vE '\buint(8|16|32|64)_t\b' | \
	grep -vE '\bint\s+main\b' | \
	grep -vE '\bint\s+argc\b' | \
	grep -vE '^\s*//' | \
	grep -vE '^\s*\*' | \
	grep -vE '#include' || true)

ALL_VIOLATIONS="${VIOLATIONS}${INT_VIOLATIONS}"

if [ -n "$ALL_VIOLATIONS" ]; then
	echo "Use stdint.h types: int->int32_t, unsigned int->uint32_t, short->int16_t, char->uint8_t" >&2
	echo "Violations:" >&2
	echo "$ALL_VIOLATIONS" | head -5 >&2
	hook_block "bare primitive type detected. Use stdint.h explicit-width types."
fi

exit 0
