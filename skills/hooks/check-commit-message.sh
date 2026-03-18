#!/bin/bash
# [PreToolUse] Bash - blocking (exit 2)
# Enforce Conventional Commits format: [OP#XXX] type(scope): description

source "$(dirname "$0")/lib/common.sh"
hook_parse_input

# Only check git commit commands
echo "$COMMAND" | grep -qE 'git\s+commit' || exit 0

# HEREDOC pattern (cat <<EOF) — skip (hard to parse)
echo "$COMMAND" | grep -q 'cat <<' && exit 0

# Extract commit message from -m "message" or -m 'message'
MSG=$(echo "$COMMAND" | grep -oE "\-m\s+[\"'][^\"']*[\"']" | sed "s/-m\s*[\"']//;s/[\"']$//" | head -1)

[ -z "$MSG" ] && exit 0

# Validate: [OP#XXX] type(scope): description
if ! echo "$MSG" | grep -qE '^\[(OP#[0-9]+)\]\s+(feat|fix|refactor|docs|style|test|chore|ci|perf|build)(\(.+\))?:\s+.+'; then
	# Also allow without OP number: type(scope): description
	if ! echo "$MSG" | grep -qE '^(feat|fix|refactor|docs|style|test|chore|ci|perf|build)(\(.+\))?:\s+.+'; then
		echo "Format: [OP#XXX] type(scope): description" >&2
		echo "Or: type(scope): description" >&2
		echo "Types: feat, fix, refactor, docs, style, test, chore, ci, perf, build" >&2
		echo "Got: $MSG" >&2
		hook_block "Commit message does not follow Conventional Commits format."
	fi
fi

exit 0
