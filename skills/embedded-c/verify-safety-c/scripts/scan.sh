#!/bin/bash
# scan.sh — Quick project scan for common C safety violations
# Usage: bash .claude/skills/verify-safety-c/scripts/scan.sh [Sources/|Drivers/|all]
# Note: This script catches surface-level patterns. Deep semantic analysis requires
# Claude to Read the code and trace control flow manually.

PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
SCOPE="${1:-all}"

if [ "$SCOPE" = "all" ]; then
    DIRS="$PROJECT_ROOT/Sources $PROJECT_ROOT/Drivers"
else
    DIRS="$PROJECT_ROOT/$SCOPE"
fi

echo "{"
echo "  \"skill\": \"verify-safety-c\","
echo "  \"scope\": \"$SCOPE\","
echo "  \"checks\": {"

# Check: Division without zero-check (surface pattern)
DIV_ZERO=$(grep -rnE '[^/]/[^/*]' $DIRS --include='*.c' 2>/dev/null | \
    grep -vE '^\s*//' | grep -vE '^\s*\*' | grep -vE '#include' | \
    grep -E '\b[a-zA-Z_][a-zA-Z0-9_]*\s*/\s*[a-zA-Z_]' | wc -l)
echo "    \"potential_div_by_var\": $DIV_ZERO,"

# Check: Unreachable code after return
DEAD_CODE=$(grep -rnE '^\s*return\s+' $DIRS --include='*.c' 2>/dev/null | wc -l)
# This is just a count — Claude should verify if code follows the return
echo "    \"return_statements\": $DEAD_CODE,"

# Check: #if 0 blocks
IF_ZERO=$(grep -rnE '^\s*#if\s+0' $DIRS --include='*.c' --include='*.h' 2>/dev/null | wc -l)
echo "    \"if_zero_blocks\": $IF_ZERO"

echo "  },"
echo "  \"note\": \"Surface-level scan. Use Claude Read+Grep for deep semantic analysis of overflow, bounds, and uninitialized vars.\""
echo "}"
