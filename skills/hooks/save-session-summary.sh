#!/bin/bash
# [Stop] - non-blocking
# Save session summary: modified files, diff stats, open TODOs

CACHE_DIR=".claude/cache"
mkdir -p "$CACHE_DIR"

# Rotate: keep only latest 20 summaries
ls -1t "$CACHE_DIR"/session-summary-*.md 2>/dev/null | tail -n +21 | xargs rm -f 2>/dev/null

SUMMARY_FILE="$CACHE_DIR/session-summary-$(date '+%Y%m%d-%H%M%S').md"

{
	echo "# Session Summary — $(date '+%Y-%m-%d %H:%M')"
	echo ""

	echo "## Modified Files"
	git diff --name-only 2>/dev/null | while IFS= read -r f; do
		echo "- \`$f\`"
	done
	git diff --cached --name-only 2>/dev/null | while IFS= read -r f; do
		echo "- \`$f\` (staged)"
	done
	echo ""

	echo "## Diff Summary"
	echo '```'
	git diff --stat 2>/dev/null || echo "(no changes)"
	echo '```'
	echo ""

	# Merge session change log if exists
	if [ -f "$CACHE_DIR/session-changes.md" ]; then
		echo "## Session Change Log"
		cat "$CACHE_DIR/session-changes.md"
		rm -f "$CACHE_DIR/session-changes.md"
		echo ""
	fi

	echo "## Open TODOs"
	TODO_COUNT=$(grep -rnE 'TODO|FIXME' Sources/ Drivers/ --include="*.c" --include="*.h" 2>/dev/null | wc -l)
	echo "Total: $TODO_COUNT items"
} > "$SUMMARY_FILE"

echo "Session summary saved: $SUMMARY_FILE" >&2

exit 0
