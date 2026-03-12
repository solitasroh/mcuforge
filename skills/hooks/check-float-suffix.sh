#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# .c/.h 파일에서 F 접미사 없는 float 리터럴 차단
# Cortex-M4 soft-float 환경에서 double 연산 방지

INPUT=$(cat)

# file_path 추출
FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"//')

# 경로가 비어있으면 통과
if [ -z "$FILE_PATH" ]; then
	exit 0
fi

# .c/.h 파일만 대상
case "$FILE_PATH" in
	*.c | *.h) ;;
	*) exit 0 ;;
esac

# 벤더 코드 제외
if echo "$FILE_PATH" | grep -qE '/(CMSIS|System)/'; then
	exit 0
fi

# new_string 또는 content 추출 (Write는 content, Edit는 new_string)
CODE=$(echo "$INPUT" | sed -n 's/.*"new_string":"\([^"]*\)".*/\1/p')
if [ -z "$CODE" ]; then
	CODE=$(echo "$INPUT" | sed -n 's/.*"content":"\([^"]*\)".*/\1/p')
fi

if [ -z "$CODE" ]; then
	exit 0
fi

# F 접미사 없는 float 리터럴 탐지
# 패턴: 숫자.숫자 또는 숫자.숫자e숫자 형태에서 F/f 접미사가 없는 것
# 제외: #include, #define 줄, 문자열 내부, 주석, 정수(소수점 없음)
VIOLATIONS=$(echo "$CODE" | grep -nE '[^a-zA-Z_]([0-9]+\.[0-9]+([eE][+-]?[0-9]+)?)[^FfLUlu]' | \
	grep -v '^\s*//' | \
	grep -v '^\s*\*' | \
	grep -v '#include' | \
	grep -v '#define.*VERSION' | \
	grep -vE '\.[0-9]+F' | \
	grep -vE '"[^"]*[0-9]+\.[0-9]+[^"]*"' || true)

if [ -n "$VIOLATIONS" ]; then
	echo "BLOCKED: float 리터럴에 F 접미사가 없습니다. Cortex-M4 soft-float에서 double 연산이 발생합니다." >&2
	echo "예: 0.0 → 0.0F, 1e-9 → 1e-9F, 3.14 → 3.14F" >&2
	echo "위반:" >&2
	echo "$VIOLATIONS" | head -5 >&2
	exit 2
fi

exit 0
