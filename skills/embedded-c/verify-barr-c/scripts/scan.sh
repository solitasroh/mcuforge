#!/bin/bash
# scan.sh — Full project scan for BARR-C:2018 rule violations
# Usage: bash .claude/skills/verify-barr-c/scripts/scan.sh [Sources/|Drivers/|all]

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SCOPE="${1:-all}"

if [ "$SCOPE" = "all" ]; then
    DIRS="$PROJECT_ROOT/Sources $PROJECT_ROOT/Drivers"
else
    DIRS="$PROJECT_ROOT/$SCOPE"
fi

ISSUES=0
echo "{"
echo "  \"skill\": \"verify-barr-c\","
echo "  \"scope\": \"$SCOPE\","
echo "  \"checks\": {"

# Check 2: Forbidden keywords (goto, auto, register)
FORBIDDEN=$(grep -rnE '\b(goto|register)\b' $DIRS --include='*.c' --include='*.h' 2>/dev/null | \
    grep -v '^\s*//' | grep -v '^\s*\*' | grep -vE '"[^"]*\b(goto|register)\b[^"]*"' | wc -l)
echo "    \"forbidden_keywords\": $FORBIDDEN,"

# Check 3: Assignment in conditions
ASSIGN_COND=$(grep -rnE '(if|while)\s*\([^)]*[^!=<>]=(?!=)[^=]' $DIRS --include='*.c' 2>/dev/null | \
    grep -v '^\s*//' | wc -l)
echo "    \"assignment_in_condition\": $ASSIGN_COND,"

# Check 5: Float equality
FLOAT_EQ=$(grep -rnE '[!=]=\s*[0-9]+\.[0-9]|[0-9]+\.[0-9]+[fF]?\s*[!=]=' $DIRS --include='*.c' 2>/dev/null | \
    grep -v '^\s*//' | grep -v '^\s*\*' | wc -l)
echo "    \"float_equality\": $FLOAT_EQ"

ISSUES=$((FORBIDDEN + ASSIGN_COND + FLOAT_EQ))

echo "  },"
echo "  \"total_issues\": $ISSUES"
echo "}"

[ $ISSUES -eq 0 ] && exit 0 || exit 1
