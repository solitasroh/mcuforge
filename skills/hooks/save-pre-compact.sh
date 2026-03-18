#!/bin/bash
# [PreCompact] - non-blocking
# Save critical context before context compression

CACHE_DIR=".claude/cache"
mkdir -p "$CACHE_DIR"

{
	echo "# Pre-Compact State — $(date '+%Y-%m-%d %H:%M')"
	echo ""
	echo "## Branch: $(git branch --show-current 2>/dev/null)"
	echo ""
	echo "## Modified Files"
	git diff --name-only 2>/dev/null | while IFS= read -r f; do
		echo "- \`$f\`"
	done
	git diff --cached --name-only 2>/dev/null | while IFS= read -r f; do
		echo "- \`$f\` (staged)"
	done
	echo ""
	echo "## Recent Commits"
	git log --oneline -5 2>/dev/null || true
	echo ""
	echo "## Open TODOs"
	TODO_COUNT=$(grep -rnE 'TODO|FIXME' Sources/ Drivers/ --include="*.c" --include="*.h" 2>/dev/null | wc -l)
	echo "Total: $TODO_COUNT items"
} > "$CACHE_DIR/pre-compact-state.md"

exit 0
