#!/usr/bin/env bash
set -euo pipefail

if (( $# < 3 || $# > 4 )) || [[ -z "$1" || -z "$2" || -z "$3" ]]; then
  echo "Usage: $0 BEFORE AFTER SOURCE [--ignore-hidden|-i]" >&2
  exit 1
fi
before=("$1")
after=("$2")
if (( $# == 4 )); then
  case "$4" in
    --ignore-hidden|-i) before+=("$4"); after+=("$4") ;;
    *) echo "Error: only --ignore-hidden or -i is supported as the fourth argument" >&2; exit 1 ;;
  esac
fi
# A relative source beginning with '-' must remain a path.
before+=(-- "$3")
after+=(-- "$3")

tmp=$(mktemp -d "${TMPDIR:-/tmp}/paq-regression.XXXXXXXX")
trap 'rm -rf -- "$tmp"' EXIT
if ! "${before[@]}" > "$tmp/before.stdout"; then
  echo "Error: before executable failed; refusing to benchmark" >&2
  exit 1
fi
if ! "${after[@]}" > "$tmp/after.stdout"; then
  echo "Error: after executable failed; refusing to benchmark" >&2
  exit 1
fi
if ! cmp -s "$tmp/before.stdout" "$tmp/after.stdout"; then
  echo "Error: stdout differs (including newline bytes); refusing to benchmark" >&2
  exit 1
fi

# Bash's %q preserves quotes, newlines and shell metacharacters. Hyperfine
# explicitly uses the matching shell to execute these serialized arguments.
printf -v before_command '%q ' "${before[@]}"
printf -v after_command '%q ' "${after[@]}"
hyperfine --shell bash --warmup 3 \
  -n "[before] $1" "$before_command" \
  -n "[after] $2" "$after_command"
