#!/bin/sh
# Install a verified GitHub release. Keep execution at the end for curl | sh.
set -eu

fail() {
  printf 'paq: %s\n' "$*" >&2
  exit 1
}

cleanup() {
  [ -z "$work_dir" ] || rm -rf "$work_dir"
  [ -z "$staged_binary" ] || rm -f "$staged_binary"
}

request() {
  curl --fail --location --silent --show-error \
    --proto '=https' --proto-redir '=https' --tlsv1.2 \
    --connect-timeout 15 --max-time 300 --retry 3 \
    "$@" || fail 'GitHub release request failed'
}

download() {
  request --output "$2" "$1"
}

select_platform() {
  os=$(uname -s)
  arch=$(uname -m)
  if [ "$os" = Darwin ] && [ "$arch" = x86_64 ]; then
    # A terminal running under Rosetta reports x86_64 on Apple Silicon.
    if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || true)" = 1 ]; then
      arch=arm64
    fi
  fi
  if [ "$os" = Linux ]; then
    case "$arch" in
    x86_64 | amd64)
      # uname describes the kernel, which may host a 32-bit userspace.
      bitness=$(getconf LONG_BIT) || fail 'cannot determine Linux userspace bitness'
      case "$bitness" in
      32) arch=i686 ;;
      64) ;;
      *) fail "unsupported Linux userspace bitness: $bitness" ;;
      esac
      ;;
    esac
  fi
  case "$os:$arch" in
  Linux:x86_64 | Linux:amd64) platform=ubuntu-x64 ;;
  Linux:i386 | Linux:i486 | Linux:i586 | Linux:i686) platform=ubuntu-x86 ;;
  Darwin:arm64 | Darwin:aarch64) platform=macos-arm64 ;;
  Darwin:x86_64) platform=macos-x64 ;;
  *) fail "no release binary for $os/$arch; see https://github.com/gregl83/paq#installation" ;;
  esac
  if [ "$os" = Linux ]; then
    getconf GNU_LIBC_VERSION >/dev/null 2>&1 ||
      fail 'Linux release binaries require glibc; use cargo install paq on other systems'
  fi
}

select_release() {
  if [ "$version" = latest ]; then
    # Resolve once so the ZIP and checksum always come from the same release.
    resolved_url=$(request --head --output /dev/null --write-out '%{url_effective}' \
      https://github.com/gregl83/paq/releases/latest)
    case "$resolved_url" in
    https://github.com/gregl83/paq/releases/tag/v*) version=${resolved_url##*/} ;;
    *) fail "unexpected latest release URL: $resolved_url" ;;
    esac
  fi
  version=${version#v}
  # Accept numeric release versions and optional prerelease/build suffixes.
  printf '%s\n' "$version" | LC_ALL=C grep -Eq \
    '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?(\+[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$' ||
    fail "invalid release version: $version"
  release_url="https://github.com/gregl83/paq/releases/download/v$version"
}

verify_archive() {
  # Match exactly one checksum for this archive; never use downloaded paths.
  expected=$(awk -v asset="$asset" 'NF == 2 && $2 == asset { print $1 }' "$work_dir/$asset.sha256")
  [ "${#expected}" -eq 64 ] || fail 'invalid checksum file'
  case "$expected" in *[!0-9a-fA-F]*) fail 'invalid checksum file' ;; esac
  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$work_dir/$asset")
  else
    actual=$(shasum -a 256 "$work_dir/$asset")
  fi
  actual=${actual%% *}
  [ "$actual" = "$expected" ] || fail 'archive checksum mismatch'
}

install_binary() {
  entries=$(unzip -Z1 "$work_dir/$asset") || fail 'invalid release archive'
  [ "$entries" = paq ] || fail 'release archive must contain only paq'
  # Stream the entry to a regular file instead of extracting archive paths.
  unzip -p "$work_dir/$asset" paq >"$work_dir/paq" || fail 'cannot unpack release'
  mkdir -p "$install_dir"
  [ ! -d "$install_dir/paq" ] || fail 'destination paq is a directory'
  staged_binary=$(mktemp "$install_dir/.paq-install.XXXXXX")
  cp "$work_dir/paq" "$staged_binary"
  chmod 0755 "$staged_binary"
  "$staged_binary" --version ||
    fail 'binary cannot run here; check OS/runtime compatibility and directory execute permissions'
  mv -f "$staged_binary" "$install_dir/paq"
  staged_binary=''
  printf 'Installed paq to %s/paq\n' "$install_dir"
  case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *) printf 'Add %s to your shell PATH to run paq.\n' "$install_dir" ;;
  esac
}

main() {
  case "${1:-}" in
  -h | --help)
    printf '%s\n' 'Usage: sh install.sh' \
      'PAQ_VERSION: release version (e.g. 2.0.0 or v2.0.0); default: latest' \
      "PAQ_INSTALL_DIR: absolute installation directory; default: \$HOME/.local/bin" \
      'Supports Linux x86/x64 (glibc) and macOS x64/arm64.'
    return
    ;;
  '') ;;
  *) fail "unknown argument: $1" ;;
  esac
  [ "$#" -eq 0 ] || fail 'unexpected arguments'
  umask 077
  work_dir=''
  staged_binary=''
  trap cleanup 0
  trap 'exit 129' HUP
  trap 'exit 130' INT
  trap 'exit 143' TERM
  version=${PAQ_VERSION:-latest}
  install_dir=${PAQ_INSTALL_DIR:-${HOME:?HOME is not set}/.local/bin}
  case "$install_dir" in
  /) fail 'refusing to install into /' ;;
  /*) ;;
  *) fail 'PAQ_INSTALL_DIR must be an absolute path' ;;
  esac
  for command in curl unzip mktemp uname grep awk cp chmod mv mkdir rm; do
    command -v "$command" >/dev/null 2>&1 || fail "required command not found: $command"
  done
  command -v sha256sum >/dev/null 2>&1 || command -v shasum >/dev/null 2>&1 ||
    fail 'sha256sum or shasum is required'
  select_platform
  select_release
  asset="paq-$platform.zip"
  work_dir=$(mktemp -d)
  printf 'Downloading paq %s for %s...\n' "$version" "$platform"
  download "$release_url/$asset" "$work_dir/$asset"
  download "$release_url/$asset.sha256" "$work_dir/$asset.sha256"
  verify_archive
  install_binary
}

main "$@"
