#!/bin/bash
# [SessionStart] - non-blocking
# 세션 시작 시 프로젝트 컨텍스트를 Claude에 주입

echo "=== Project Context ==="

# 현재 브랜치 및 OP 번호
BRANCH=$(git branch --show-current 2>/dev/null)
if [ -n "$BRANCH" ]; then
	echo "Branch: $BRANCH"
	OP_NUM=$(echo "$BRANCH" | grep -oE 'OP-[0-9]+' || true)
	if [ -n "$OP_NUM" ]; then
		echo "Issue: $OP_NUM"
	fi
fi

# 최근 3개 커밋
echo ""
echo "Recent commits:"
git log --oneline -3 2>/dev/null || true

# 마지막 빌드 결과 (Flash/RAM)
ELF_FILE="output/a2750irm.elf"
if [ -f "$ELF_FILE" ]; then
	echo ""
	echo "Last build size:"
	arm-none-eabi-size "$ELF_FILE" 2>/dev/null || true
fi

# 현재 수정된 파일
MODIFIED=$(git status --short 2>/dev/null)
if [ -n "$MODIFIED" ]; then
	echo ""
	echo "Modified files:"
	echo "$MODIFIED"
fi

# TODO/FIXME 수
TODO_COUNT=$(grep -rnE 'TODO|FIXME|HACK|XXX' Sources/ Drivers/ --include="*.c" --include="*.h" 2>/dev/null | wc -l)
if [ "$TODO_COUNT" -gt 0 ]; then
	echo ""
	echo "Open TODO/FIXME: $TODO_COUNT items"
fi

echo "=== End Context ==="
exit 0
