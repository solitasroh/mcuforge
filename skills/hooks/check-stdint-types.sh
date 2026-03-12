#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# .c/.h 파일에서 int, unsigned char 등 원시 타입 사용 차단
# stdint.h 명시적 크기 타입(uint8_t, int32_t 등) 사용 강제

INPUT=$(cat)

FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"//')

if [ -z "$FILE_PATH" ]; then
	exit 0
fi

case "$FILE_PATH" in
	*.c | *.h) ;;
	*) exit 0 ;;
esac

# 벤더 코드 제외
if echo "$FILE_PATH" | grep -qE '/(CMSIS|System)/'; then
	exit 0
fi

CODE=$(echo "$INPUT" | sed -n 's/.*"new_string":"\([^"]*\)".*/\1/p')
if [ -z "$CODE" ]; then
	CODE=$(echo "$INPUT" | sed -n 's/.*"content":"\([^"]*\)".*/\1/p')
fi

if [ -z "$CODE" ]; then
	exit 0
fi

# 원시 타입 탐지 (변수 선언/함수 파라미터 위치에서)
# 허용: main 함수의 int 반환, sizeof 표현식, 캐스트 내 타입
# 차단: int, short, long, unsigned int, unsigned char, signed int 등
VIOLATIONS=$(echo "$CODE" | grep -nE '\b(unsigned\s+)?(short|long)\b|\bunsigned\s+char\b|\bsigned\s+(int|char)\b' || true)

# 단독 int 탐지 (int main, int argc 등 허용, 변수 선언의 int 차단)
# int32_t, uint8_t 등은 통과, 단독 int만 차단
INT_VIOLATIONS=$(echo "$CODE" | grep -nE '\bint\b' | \
	grep -vE '\bint(8|16|32|64)_t\b' | \
	grep -vE '\buint(8|16|32|64)_t\b' | \
	grep -vE '\bint\s+main\b' | \
	grep -vE '\bint\s+argc\b' | \
	grep -vE '^\s*//' | \
	grep -vE '^\s*\*' | \
	grep -vE '#include' || true)

ALL_VIOLATIONS="${VIOLATIONS}${INT_VIOLATIONS}"

if [ -n "$ALL_VIOLATIONS" ]; then
	echo "BLOCKED: 원시 타입 사용이 감지되었습니다. stdint.h 타입을 사용하세요." >&2
	echo "변환: int → int32_t, unsigned int → uint32_t, short → int16_t, char → uint8_t" >&2
	echo "위반:" >&2
	echo "$ALL_VIOLATIONS" | head -5 >&2
	exit 2
fi

exit 0
