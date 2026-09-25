# Releasing paq

For end-user installation instructions, see the [installation guide](install.md).

## Publishing a release

Push a version tag such as `v2.0.0` pointing at the intended release commit to
start the [CD workflow](../.github/workflows/cd.yml). Let the workflow
publish the release; do not manually publish an empty release first.

Ensure `Cargo.toml` has the matching version, then tag the release commit:

```bash
git tag -a v2.0.0 <release-commit> -m "v2.0.0"
git push origin v2.0.0
```

The GitHub release title matches the tag, such as `v2.0.0`, consistent with past releases.

The workflow builds all six platform binaries and packages each as a ZIP with a
matching `.zip.sha256` file. All packaged binaries must pass smoke tests before
publication.

[bin/publish-release.sh](../bin/publish-release.sh) verifies the complete local
asset set, creates or resumes a draft, uploads all twelve assets, checks that they
are present, and publishes the release. Prerelease tags remain prereleases.
The workflow then publishes the crate to crates.io.

Failed uploads leave the release as a draft. Rerun the failed workflow jobs to
retry. The publishing script refuses to overwrite published releases.

## Release smoke tests

[bin/smoke-release.py](../bin/smoke-release.py) checks each packaged binary's
checksum, executable architecture, release version, and known file-hash output.

macOS uses separate Intel and ARM64 runners. Linux x86 and Windows x86 binaries
run with the runners' 32-bit compatibility support. Native Linux x64 and macOS
tests run the shell installer with only its download command replaced to read
unpublished assets locally. Windows and Linux x86 on a 64-bit runner test manual
extraction and execution.

## Installer tests

[bin/test-installer.py](../bin/test-installer.py) uses local ZIP fixtures and
substitutes network and platform detection commands. It covers platform and
version selection, Rosetta, 32-bit userspace, paths with spaces, checksum and
archive failures, startup failures, and preservation of existing installations.
CI runs these tests on Linux and macOS.

On Linux, the suite also tests publication ordering and failures with a mock
GitHub CLI. Tests never create or publish a real release.

Run from the repository root:

```bash
sh -n install.sh
bash -n bin/publish-release.sh
python3 bin/test-installer.py
```
