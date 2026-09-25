#!/usr/bin/env bash

set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Error: 'before' executable path is required" >&2
  exit 1
fi
BEFORE_PAC_PATH="$1"

if [ -z "${2:-}" ]; then
  echo "Error: 'after' executable path is required" >&2
  exit 1
fi
AFTER_PAC_PATH="$2"

if [ -z "${3:-}" ]; then
  echo "Error: target data source system path to hash is required" >&2
  exit 1
fi
TARGET_PATH="$3"
shift 3

printf -v BEFORE_COMMAND '%q ' "$BEFORE_PAC_PATH" "$TARGET_PATH"
printf -v AFTER_COMMAND '%q ' "$AFTER_PAC_PATH" "$TARGET_PATH"

hyperfine \
  --shell bash \
  -n "[before] ${BEFORE_PAC_PATH}" "$BEFORE_COMMAND" \
  -n "[after] ${AFTER_PAC_PATH}" "$AFTER_COMMAND" \
  --warmup 3 "$@"
