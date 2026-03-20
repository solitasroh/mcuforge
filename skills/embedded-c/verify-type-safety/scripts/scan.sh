#!/bin/bash
# scan.sh — Full project scan for bare primitive types (int, short, long, unsigned char)
# Usage: bash .claude/skills/verify-type-safety/scripts/scan.sh [Sources/|Drivers/|all]

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SCOPE="${1:-all}"

if [ "$SCOPE" = "all" ]; then
    DIRS="$PROJECT_ROOT/Sources $PROJECT_ROOT/Drivers $PROJECT_ROOT/Components"
else
    DIRS="$PROJECT_ROOT/$SCOPE"
fi

ISSUES=0
echo "{"
echo "  \"skill\": \"verify-type-safety\","
echo "  \"scope\": \"$SCOPE\","
echo "  \"issues\": ["

FIRST=true
for dir in $DIRS; do
    [ -d "$dir" ] || continue
    # Bare primitive types
    while IFS= read -r line; do
        file=$(echo "$line" | cut -d: -f1)
        lineno=$(echo "$line" | cut -d: -f2)
        relpath="${file#$PROJECT_ROOT/}"
        if [ "$FIRST" = true ]; then FIRST=false; else echo ","; fi
        printf '    {"file": "%s", "line": %s, "type": "bare_primitive"}' "$relpath" "$lineno"
        ISSUES=$((ISSUES + 1))
    done < <(grep -rnE '\b(unsigned int|signed int|unsigned long|signed long|long long|short|unsigned char|signed char)\b' "$dir" --include='*.c' --include='*.h' 2>/dev/null | \
        grep -v '^\s*//' | grep -v '^\s*\*' || true)

    # Bare 'int' (excluding int main, uint*, int32_t etc.)
    while IFS= read -r line; do
        file=$(echo "$line" | cut -d: -f1)
        lineno=$(echo "$line" | cut -d: -f2)
        relpath="${file#$PROJECT_ROOT/}"
        if [ "$FIRST" = true ]; then FIRST=false; else echo ","; fi
        printf '    {"file": "%s", "line": %s, "type": "bare_int"}' "$relpath" "$lineno"
        ISSUES=$((ISSUES + 1))
    done < <(grep -rnE '\bint\b\s+[a-zA-Z_]' "$dir" --include='*.c' --include='*.h' 2>/dev/null | \
        grep -v 'int main' | grep -v 'uint' | grep -v 'int[0-9]' | \
        grep -v '^\s*//' | grep -v '^\s*\*' || true)
done

echo ""
echo "  ],"
echo "  \"total_issues\": $ISSUES"
echo "}"

[ $ISSUES -eq 0 ] && exit 0 || exit 1
