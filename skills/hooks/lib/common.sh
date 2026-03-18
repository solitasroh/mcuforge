#!/bin/bash
# .claude/hooks/lib/common.sh — hook common utilities
# All hooks source this file for JSON parsing, filtering, and output helpers.

# Parse stdin JSON via jq (with legacy grep/sed fallback)
hook_parse_input() {
	INPUT=$(cat)
	if command -v jq &>/dev/null; then
		FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')
		CODE=$(echo "$INPUT" | jq -r '.tool_input.new_string // .tool_input.content // empty')
		COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')
	else
		FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | sed 's/"file_path":"//;s/"//')
		CODE=$(echo "$INPUT" | sed -n 's/.*"new_string":"\([^"]*\)".*/\1/p')
		[ -z "$CODE" ] && CODE=$(echo "$INPUT" | sed -n 's/.*"content":"\([^"]*\)".*/\1/p')
		COMMAND=$(echo "$INPUT" | grep -o '"command":"[^"]*"' | sed 's/"command":"//;s/"//')
	fi
	export FILE_PATH CODE COMMAND INPUT
}

# Require non-empty file_path (exit 0 = skip silently)
hook_require_filepath() {
	[ -z "$FILE_PATH" ] && exit 0
	return 0
}

# Require .c or .h extension
hook_require_c_file() {
	case "$FILE_PATH" in
		*.c | *.h) ;;
		*) exit 0 ;;
	esac
}

# Skip CMSIS/System vendor code
hook_skip_vendor() {
	echo "$FILE_PATH" | grep -qE '(^|/)CMSIS/|(^|/)System/' && exit 0
	return 0
}

# Require non-empty CODE
hook_require_code() {
	[ -z "$CODE" ] && exit 0
	return 0
}

# Emit warning via additionalContext (Claude sees it, no blocking)
hook_warn() {
	local msg="$1"
	if command -v jq &>/dev/null; then
		jq -n --arg m "WARNING: $msg" \
			'{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:$m}}'
	else
		echo "WARNING: $msg" >&2
	fi
	exit 0
}

# Block with stderr feedback to Claude (exit 2)
hook_block() {
	echo "BLOCKED: $1" >&2
	exit 2
}
