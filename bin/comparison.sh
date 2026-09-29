#!/usr/bin/env bash

set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Error: target data source system path to hash is required" >&2
  exit 1
fi
printf -v TARGET_PATH '%q' "$1"
shift

# Keep an explicit run count supplied by the caller (including smoke runs).
RUN_ARGS=(--runs 20)
for arg in "$@"; do
  case "$arg" in
  -r | -r[0-9]* | -r=* | --runs | --runs=*) RUN_ARGS=() ;;
  esac
done

hyperfine \
  --shell "bash -o pipefail" \
  "paq ${TARGET_PATH}" \
  "merkle-hash ${TARGET_PATH}" \
  "directory-checksum --max-depth=0 ${TARGET_PATH}" \
  "hashrat -sha256 -dir -hidden ${TARGET_PATH}" \
  "fd -HI -tf . ${TARGET_PATH} -0 | xargs -0 -P0 b3sum | LC_ALL=C sort | b3sum" \
  "fd -HI -tf . ${TARGET_PATH} -0 | xargs -0 -P0 sha256sum | LC_ALL=C sort | sha256sum" \
  "fd -HI -tf . ${TARGET_PATH} -0 | xargs -0 -P0 md5sum | LC_ALL=C sort | md5sum" \
  "dirhash ${TARGET_PATH} -a sha256" \
  "checksumdir -a sha256 ${TARGET_PATH}" \
  "folder-hash ${TARGET_PATH}" \
  --warmup 3 "${RUN_ARGS[@]}" "$@"
