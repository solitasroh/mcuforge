#!/bin/bash
# [SessionStart] - non-blocking
# Inject project context at session start + clean old cache files

# Clean old session-summary files (keep latest 20)
CACHE_DIR=".claude/cache"
if [ -d "$CACHE_DIR" ]; then
	ls -1t "$CACHE_DIR"/session-summary-*.md 2>/dev/null | tail -n +21 | xargs rm -f 2>/dev/null
fi

echo "=== Project Context ==="

# Current branch and OP number
BRANCH=$(git branch --show-current 2>/dev/null)
if [ -n "$BRANCH" ]; then
	echo "Branch: $BRANCH"
	OP_NUM=$(echo "$BRANCH" | grep -oE 'OP-[0-9]+' || true)
	if [ -n "$OP_NUM" ]; then
		echo "Issue: $OP_NUM"
	fi
fi

# Recent 3 commits
echo ""
echo "Recent commits:"
git log --oneline -3 2>/dev/null || true

# Last build size (find ELF file dynamically)
ELF_FILE=$(find output/ -name "*.elf" -type f 2>/dev/null | head -1)
if [ -n "$ELF_FILE" ] && [ -f "$ELF_FILE" ]; then
	echo ""
	echo "Last build size:"
	arm-none-eabi-size "$ELF_FILE" 2>/dev/null || true
fi

# Modified files
MODIFIED=$(git status --short 2>/dev/null)
if [ -n "$MODIFIED" ]; then
	echo ""
	echo "Modified files:"
	echo "$MODIFIED"
fi

# TODO/FIXME count
TODO_COUNT=$(grep -rnE 'TODO|FIXME|HACK|XXX' Sources/ Drivers/ --include="*.c" --include="*.h" 2>/dev/null | wc -l)
if [ "$TODO_COUNT" -gt 0 ]; then
	echo ""
	echo "Open TODO/FIXME: $TODO_COUNT items"
fi

echo "=== End Context ==="
exit 0
