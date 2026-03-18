#!/bin/bash
# [PostToolUse] Write|Edit - non-blocking
# Track TODO/FIXME/HACK/XXX comments in edited files

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor

# Search current file for TODO markers
TODOS=$(grep -nE 'TODO|FIXME|HACK|XXX' "$FILE_PATH" 2>/dev/null || true)

[ -z "$TODOS" ] && exit 0

CACHE_DIR=".claude/cache"
mkdir -p "$CACHE_DIR"

LOG_FILE="$CACHE_DIR/todo-log.md"

{
	echo ""
	echo "### $(date '+%Y-%m-%d %H:%M') — $FILE_PATH"
	echo "$TODOS" | while IFS= read -r line; do
		echo "- $line"
	done
} >> "$LOG_FILE"

TODO_COUNT=$(echo "$TODOS" | wc -l)
echo "INFO: $FILE_PATH: $TODO_COUNT TODO/FIXME items logged to $LOG_FILE" >&2

exit 0
