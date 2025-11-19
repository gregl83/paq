#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "Error: 'before' `paq` executable path is required" >&2
  exit 1
fi
BEFORE_PAC_PATH="$1"

if [ -z "$2" ]; then
  echo "Error: 'after' `paq` executable path is required" >&2
  exit 1
fi
AFTER_PAC_PATH="$2"

if [ -z "$3" ]; then
  echo "Error: target data source system path to hash is required" >&2
  exit 1
fi
TARGET_PATH="$3"

hyperfine \
  -n "[before] ${BEFORE_PAC_PATH}" "${BEFORE_PAC_PATH} '${TARGET_PATH}'" \
  -n "[after] ${AFTER_PAC_PATH}" "${AFTER_PAC_PATH} '${TARGET_PATH}'" \
  --warmup 3
