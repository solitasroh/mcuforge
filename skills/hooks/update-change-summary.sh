#!/bin/bash
# [PostToolUse] Write|Edit - non-blocking
# Record modified files to session change log

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor

CACHE_DIR=".claude/cache"
mkdir -p "$CACHE_DIR"

SUMMARY_FILE="$CACHE_DIR/session-changes.md"

# Create header on first entry
if [ ! -f "$SUMMARY_FILE" ]; then
	echo "# Session Changes — $(date '+%Y-%m-%d %H:%M')" > "$SUMMARY_FILE"
	echo "" >> "$SUMMARY_FILE"
fi

# Deduplicate: record each file only once
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
RELATIVE_PATH=$(echo "$FILE_PATH" | sed "s|${PROJECT_DIR}/||")
TIMESTAMP=$(date '+%H:%M:%S')

if ! grep -q "$RELATIVE_PATH" "$SUMMARY_FILE" 2>/dev/null; then
	echo "- \`$RELATIVE_PATH\` — first modified at $TIMESTAMP" >> "$SUMMARY_FILE"
fi

exit 0
