#!/usr/bin/env bash

set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: $0 <corpus-directory>" >&2
  exit 1
fi

TARGET_PATH=$(realpath -- "$1")
WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT

# Absolute paths make the two discovery tools' path spelling comparable.
find "$TARGET_PATH" -type f -print0 | LC_ALL=C sort -z >"$WORK_DIR/find-files"
fd -HI -tf . "$TARGET_PATH" -0 | LC_ALL=C sort -z >"$WORK_DIR/fd-files"
cmp "$WORK_DIR/find-files" "$WORK_DIR/fd-files"
echo "PASS: fd and find select identical files."

for tool in b3sum md5sum sha256sum; do
  # The serial reference also sorts checksum records, not input paths alone.
  xargs -0 "$tool" <"$WORK_DIR/find-files" | LC_ALL=C sort | "$tool" >"$WORK_DIR/reference"
  for run in 1 2 3; do
    fd -HI -tf . "$TARGET_PATH" -0 | xargs -0 -P0 "$tool" | LC_ALL=C sort | "$tool" >"$WORK_DIR/parallel-$run"
    cmp "$WORK_DIR/reference" "$WORK_DIR/parallel-$run"
  done
  echo "PASS: $tool matches the serial reference in all three parallel runs."
done
