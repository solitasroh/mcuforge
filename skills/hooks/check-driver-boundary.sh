#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# Drivers/ 파일이 Sources/ 헤더를 include하는 역참조 차단
# 계층 규칙: Sources/ → Drivers/ (OK), Drivers/ → Sources/ (차단)

INPUT=$(cat)

FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"//')

if [ -z "$FILE_PATH" ]; then
	exit 0
fi

# Drivers/ 내 파일만 검사
if ! echo "$FILE_PATH" | grep -qE '/Drivers/'; then
	exit 0
fi

case "$FILE_PATH" in
	*.c | *.h) ;;
	*) exit 0 ;;
esac

CODE=$(echo "$INPUT" | sed -n 's/.*"new_string":"\([^"]*\)".*/\1/p')
if [ -z "$CODE" ]; then
	CODE=$(echo "$INPUT" | sed -n 's/.*"content":"\([^"]*\)".*/\1/p')
fi

if [ -z "$CODE" ]; then
	exit 0
fi

# Sources/ 헤더를 include하는지 검사
# 패턴: #include "measure.h", #include "calibration.h" 등 Sources/ 파일 참조
SOURCES_HEADERS=$(ls Sources/*.h 2>/dev/null | xargs -I{} basename {} 2>/dev/null)

VIOLATIONS=""
for header in $SOURCES_HEADERS; do
	if echo "$CODE" | grep -qE "#include\s+\"$header\""; then
		VIOLATIONS="${VIOLATIONS}  #include \"$header\"\n"
	fi
done

if [ -n "$VIOLATIONS" ]; then
	echo "BLOCKED: Drivers/ 파일이 Sources/ 헤더를 참조합니다 (계층 역참조)." >&2
	echo "규칙: Drivers/ → Sources/ 방향 의존은 금지됩니다." >&2
	echo "위반:" >&2
	echo -e "$VIOLATIONS" >&2
	exit 2
fi

exit 0
