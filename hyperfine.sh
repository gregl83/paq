#!/usr/bin/env bash

hyperfine \
  "paq ./go" \
  "dirhash -a sha256 ./go" \
  "find ./go -type f -print0 | LC_ALL=C sort -z | xargs -0 b3sum | b3sum" \
  "find ./go -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum" \
  --warmup 3 \
  --export-markdown paq.md
