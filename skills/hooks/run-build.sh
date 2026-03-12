#!/bin/bash
# [PostToolUse] Write|Edit - non-blocking (async)
# .c/.h 파일 수정 후 cmake --build --preset Release 실행
# 빌드 실패 시 에러 메시지 출력 (non-blocking이므로 exit 0)

INPUT=$(cat)

FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"//')

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

# cmake 존재 확인
if ! command -v cmake &>/dev/null; then
	exit 0
fi

# 빌드 프리셋 설정 파일 확인
if [ ! -f CMakePresets.json ]; then
	exit 0
fi

# 빌드 실행 (에러 출력만, non-blocking이라 exit 0)
BUILD_OUTPUT=$(cmake --build --preset Release 2>&1)
BUILD_EXIT=$?

if [ $BUILD_EXIT -ne 0 ]; then
	echo "BUILD FAILED:" >&2
	echo "$BUILD_OUTPUT" | tail -20 >&2
fi

exit 0
