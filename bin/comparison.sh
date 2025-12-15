#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "Error: target data source system path to hash is required" >&2
  exit 1
fi
TARGET_PATH="$1"

hyperfine \
  "paq ${TARGET_PATH}" \
  "find ${TARGET_PATH} -type f -print0 | LC_ALL=C sort -z | xargs -0 b3sum | b3sum" \
  "find ${TARGET_PATH} -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum" \
  "dirhash ${TARGET_PATH} -a sha256" \
  "folder-hash ${TARGET_PATH}" \
  --warmup 3
