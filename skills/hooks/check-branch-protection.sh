#!/bin/bash
# [PreToolUse] Write|Edit - blocking (exit 2)
# Block direct file edits on main/master branch

BRANCH=$(git branch --show-current 2>/dev/null)

if [[ "$BRANCH" == "main" || "$BRANCH" == "master" ]]; then
	echo "Create a feature branch first: git checkout -b feature/OP-XXX-description" >&2
	echo "BLOCKED: Direct editing on $BRANCH branch is not allowed." >&2
	exit 2
fi

exit 0
