#!/usr/bin/env bash
# Auto-format Rust files after Edit/Write tool calls.
# Receives hook context as JSON on stdin.

FILE_PATH=$(jq -r '.tool_input.file_path // empty')

# Only format Rust files
[ "${FILE_PATH##*.}" = "rs" ] || exit 0

rustfmt "$FILE_PATH" 2>/dev/null
exit 0
