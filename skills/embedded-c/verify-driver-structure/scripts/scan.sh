#!/bin/bash
# scan.sh — Full project scan for driver structure violations
# Usage: bash .claude/skills/verify-driver-structure/scripts/scan.sh

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
DRIVERS="$PROJECT_ROOT/Drivers"

ISSUES=0
echo "{"
echo "  \"skill\": \"verify-driver-structure\","
echo "  \"issues\": ["

FIRST=true

# Check 1: Reverse dependency (Drivers/ including Sources/ or Components/ headers)
while IFS= read -r line; do
    file=$(echo "$line" | cut -d: -f1)
    lineno=$(echo "$line" | cut -d: -f2)
    relpath="${file#$PROJECT_ROOT/}"
    if [ "$FIRST" = true ]; then FIRST=false; else echo ","; fi
    printf '    {"file": "%s", "line": %s, "type": "reverse_dependency"}' "$relpath" "$lineno"
    ISSUES=$((ISSUES + 1))
done < <(grep -rnE '#include\s+["<](Sources|Components)/' "$DRIVERS" --include='*.c' --include='*.h' 2>/dev/null || true)

# Also check for known Sources/ header basenames
SOURCES_HEADERS=$(find "$PROJECT_ROOT/Sources" -name "*.h" -exec basename {} \; 2>/dev/null | sort -u)
for header in $SOURCES_HEADERS; do
    while IFS= read -r line; do
        file=$(echo "$line" | cut -d: -f1)
        lineno=$(echo "$line" | cut -d: -f2)
        relpath="${file#$PROJECT_ROOT/}"
        if [ "$FIRST" = true ]; then FIRST=false; else echo ","; fi
        printf '    {"file": "%s", "line": %s, "type": "reverse_dependency", "header": "%s"}' "$relpath" "$lineno" "$header"
        ISSUES=$((ISSUES + 1))
    done < <(grep -rnE "#include\s+\"$header\"" "$DRIVERS" --include='*.c' --include='*.h' 2>/dev/null || true)
done

# Check 2: Missing _init function
for file in "$DRIVERS"/*.c; do
    [ -f "$file" ] || continue
    basename=$(basename "$file" .c)
    if ! grep -q "${basename}_init" "$file" 2>/dev/null; then
        relpath="${file#$PROJECT_ROOT/}"
        if [ "$FIRST" = true ]; then FIRST=false; else echo ","; fi
        printf '    {"file": "%s", "type": "missing_init", "expected": "%s_init"}' "$relpath" "$basename"
        ISSUES=$((ISSUES + 1))
    fi
done

echo ""
echo "  ],"
echo "  \"total_issues\": $ISSUES"
echo "}"

[ $ISSUES -eq 0 ] && exit 0 || exit 1
