#!/bin/bash
# scan.sh — Full project scan for float literals missing F suffix
# Usage: bash .claude/skills/verify-float-suffix/scripts/scan.sh [Sources/|Drivers/|all]
# Output: JSON-formatted results to stdout

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SCOPE="${1:-all}"

if [ "$SCOPE" = "all" ]; then
    DIRS="$PROJECT_ROOT/Sources $PROJECT_ROOT/Drivers"
else
    DIRS="$PROJECT_ROOT/$SCOPE"
fi

ISSUES=0
echo "{"
echo "  \"skill\": \"verify-float-suffix\","
echo "  \"scope\": \"$SCOPE\","
echo "  \"issues\": ["

FIRST=true
for dir in $DIRS; do
    [ -d "$dir" ] || continue
    # Decimal literals without F/f suffix
    while IFS= read -r line; do
        file=$(echo "$line" | cut -d: -f1)
        lineno=$(echo "$line" | cut -d: -f2)
        content=$(echo "$line" | cut -d: -f3-)
        relpath="${file#$PROJECT_ROOT/}"
        if [ "$FIRST" = true ]; then FIRST=false; else echo ","; fi
        printf '    {"file": "%s", "line": %s, "code": "%s"}' "$relpath" "$lineno" "$(echo "$content" | sed 's/"/\\"/g' | tr -d '\n')"
        ISSUES=$((ISSUES + 1))
    done < <(grep -rnE '[^a-zA-Z_]([0-9]+\.[0-9]+([eE][+-]?[0-9]+)?)[^FfLUlu0-9.eE]' "$dir" --include='*.c' --include='*.h' 2>/dev/null | \
        grep -v '^\s*//' | grep -v '^\s*\*' | grep -v '#include' | \
        grep -v '#define.*VERSION' | grep -vE '\.[0-9]+[Ff]' | \
        grep -vE '"[^"]*[0-9]+\.[0-9]+[^"]*"' || true)
done

echo ""
echo "  ],"
echo "  \"total_issues\": $ISSUES"
echo "}"

[ $ISSUES -eq 0 ] && exit 0 || exit 1
