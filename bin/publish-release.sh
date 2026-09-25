#!/usr/bin/env bash
# Run from the directory containing all six release ZIPs and their checksums.
set -euo pipefail

release_tag="${1:?release tag is required}"
if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]]; then
  echo "Invalid release tag: $release_tag" >&2
  exit 1
fi

assets=()
for platform in ubuntu-x64 ubuntu-x86 macos-x64 macos-arm64 windows-x64 windows-x86; do
  archive="paq-$platform.zip"
  test -f "$archive"
  test -f "$archive.sha256"
  sha256sum --check --strict "$archive.sha256"
  assets+=("$archive" "$archive.sha256")
done

# A retry may resume an incomplete draft, but must never replace public assets.
if is_draft=$(gh release view "$release_tag" --json isDraft --jq .isDraft); then
  if [[ "$is_draft" != true ]]; then
    echo "Release $release_tag is already published; refusing to modify it." >&2
    exit 1
  fi
else
  gh release create "$release_tag" --title "$release_tag" --verify-tag --draft --generate-notes
fi

gh release upload "$release_tag" "${assets[@]}" --clobber
uploaded=$(gh release view "$release_tag" --json assets --jq '.assets[].name')
for asset in "${assets[@]}"; do
  if ! grep -Fxq "$asset" <<<"$uploaded"; then
    echo "Release is missing $asset; leaving it as a draft." >&2
    exit 1
  fi
done

prerelease=false
if [[ "${release_tag%%+*}" == *-* ]]; then
  prerelease=true
fi
gh release edit "$release_tag" --title "$release_tag" --draft=false --prerelease="$prerelease"
