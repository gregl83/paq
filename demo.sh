#!/usr/bin/env bash

set -euo pipefail

# Requires the project Rust toolchain and stable Rust for the recording tools.
REPO_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TOOLS_DIR="$REPO_DIR/target/demo-tools"

cd "$REPO_DIR"
cargo build --release --locked
cargo +stable install --locked --root "$TOOLS_DIR" \
  --git https://github.com/k9withabone/autocast \
  --rev b351ecaf550457dabb7e01ef65360fc5949cb100
cargo +stable install --locked --root "$TOOLS_DIR" \
  --git https://github.com/asciinema/agg \
  --rev aae34b012382536c8c01c753e4b609424642a06d

# Record in an isolated fixture; never modify the repository's example files.
DEMO_DIR="$(mktemp -d "$REPO_DIR/target/paq-demo.XXXXXX")"
trap 'rm -rf -- "$DEMO_DIR"' EXIT
cp -R "$REPO_DIR/example" "$DEMO_DIR/example"

cd "$DEMO_DIR"
"$TOOLS_DIR/bin/autocast" \
  --environment "PATH=$REPO_DIR/target/release:$PATH" \
  "$REPO_DIR/demo.yaml" demo.cast
"$TOOLS_DIR/bin/agg" demo.cast demo.gif
mv demo.cast "$REPO_DIR/demo.cast"
mv demo.gif "$REPO_DIR/paq.gif"
