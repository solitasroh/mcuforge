#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# Drivers/ must not include Sources/ headers (layer violation)

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath

# Only check files inside Drivers/
echo "$FILE_PATH" | grep -qE '(^|/)Drivers/' || exit 0

hook_require_c_file
hook_require_code

# Collect all Sources/ header basenames
SOURCES_HEADERS=$(find Sources/ -name "*.h" -exec basename {} \; 2>/dev/null || true)

VIOLATIONS=""
for header in $SOURCES_HEADERS; do
	if echo "$CODE" | grep -qE "#include\s+\"$header\""; then
		VIOLATIONS="${VIOLATIONS}  #include \"$header\"\n"
	fi
done

if [ -n "$VIOLATIONS" ]; then
	echo "Rule: Drivers/ -> Sources/ dependency is forbidden." >&2
	echo "Violations:" >&2
	echo -e "$VIOLATIONS" >&2
	hook_block "Drivers/ file includes Sources/ header (layer violation)."
fi

exit 0
