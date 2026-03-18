#!/bin/bash
# [PostToolUse] Write|Edit - non-blocking
# Auto-apply clang-format on .c/.h files after edit

source "$(dirname "$0")/lib/common.sh"
hook_parse_input
hook_require_filepath
hook_require_c_file
hook_skip_vendor

# Require clang-format
command -v clang-format &>/dev/null || exit 0

clang-format -i --style=file "$FILE_PATH" 2>/dev/null

exit 0
