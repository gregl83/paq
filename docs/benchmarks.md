[![Build](https://github.com/gregl83/paq/actions/workflows/build.yml/badge.svg)](https://github.com/gregl83/paq/actions/workflows/build.yml)
[![Coverage Status](https://codecov.io/gh/gregl83/paq/graph/badge.svg?token=CL93O7DW9C)](https://codecov.io/gh/gregl83/paq)
[![Crates.io](https://img.shields.io/crates/v/paq.svg)](https://crates.io/crates/paq)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/gregl83/paq/blob/master/LICENSE)

# [paq](/) / Benchmarks

This document outlines the process for creating reproducible and comparative benchmarks for `paq`.

Benchmark executable scripts are located in the [bin](../bin) directory of this repository.

Reproducibility relies on four main tools:

- **[AWS EC2](https://aws.amazon.com/ec2/):** For a consistent compute environment.
- **[Git](https://git-scm.com/):** For hash-backed data source snapshots.
- **[Nix](https://nixos.org/):** For pinned software versions and dependencies.
- **[Hyperfine](https://github.com/sharkdp/hyperfine):** For standardizing benchmark execution.

## Benchmark Tool Usage

### AWS EC2 Instance

To prioritize accessibility and ease of reproduction, benchmarks are executed on cloud computing infrastructure rather than bare-metal hardware.

[EC2](https://aws.amazon.com/ec2/) serves as the compute provider. To ensure result consistency across runs, the instance is provisioned with a strict configuration via [CloudFormation](https://docs.aws.amazon.com/cloudformation/).

The environment can be reproduced by creating a new CloudFormation stack using the [ec-benchmark-template.yaml](../infra/ec2-benchmark-template.yaml) template.

### Git Data Source Snapshot

The [Go](https://github.com/golang/go) programming language repository serves as the data source for directory hashing benchmarks.

To ensure consistency, the benchmarks target the specific version tag [go1.25.0](https://github.com/golang/go/releases/tag/go1.25.0) (commit hash: `6e676ab2b809d46623acb5988248d95d1eb7939c`). Clone and checkout steps are defined in the [ec2-benchmark-template.yaml](../infra/ec2-benchmark-template.yaml) template.

> **Note:** The data source repository's `.git` directory is deleted prior to execution. This eliminates variability caused by version control metadata.

### Nix Package Manager

The [Nix](https://search.nixos.org/packages) package manager is used to pin software builds and third-party tools to specific, hash-verified versions.

The benchmark compute instance relies on the `paq` [flake.nix](../flake.nix) configuration to set up the environment.

### Hyperfine

Benchmarks are executed using [hyperfine](https://github.com/sharkdp/hyperfine).

The [bin](../bin) directory contains a helper script, [comparison.sh](../bin/comparison.sh), which invokes `hyperfine` to run comparative benchmarks against other tools.

Hyperfine benchmark commands starting with `find` use the following command with various `<hashsum>` implementations:

```bash
find ./go -type f -print0 | LC_ALL=C sort -z | xargs -0 <hashsum> | <hashsum>
```

## Regression Testing

Benchmarks are used to ensure that `paq` release candidates have equal or better performance than past releases.

## Compatibility gate and measurement scopes

The optimization compatibility reference is revision
`300a911ffccd8f8c4bcb5e6ab17ddec6c5b07cf1`. Preserve its `Cargo.lock`, checkout,
release executable and build metadata outside the measured source tree. Build
reference and candidate with the same compiler, target, release profile and flags.
The F01 measurement repair changes benchmark/test tooling only; it makes no paq
runtime speedup claim.

The existing three-argument regression command now runs both executables
successfully and compares stdout **as bytes**, including trailing newlines,
before Hyperfine may run. Paths remain separate arguments, even when they include
spaces, quotes, newlines, shell metacharacters or a leading-dash source name.

```bash
bin/regression.sh /absolute/reference/paq /absolute/candidate/paq /data/corpus
bin/regression.sh /absolute/reference/paq /absolute/candidate/paq /data/corpus --ignore-hidden
```

`-i` is also accepted as the optional fourth argument. Run each mode on the same
unchanged fixture. The script requires Bash, `cmp`, `mktemp` and Hyperfine. It
serializes commands with Bash's `%q` and explicitly selects Bash in Hyperfine.
A mismatch or unsuccessful preflight aborts timing and removes temporary files.
Output-file behavior (`-o`/`--out`) is tested separately: it can modify the tree
being measured and is unsuitable as the normal repeated timing workload.

`cargo test --locked` includes frozen existing golden hashes and an independent
explicit-entry format reference in `tests/compatibility.rs`. It covers root and
hidden-root behavior, binary/non-ASCII content, duplicate entry digests, hard
links, entry counts around 100-entry batches, read boundaries, configured 1/2/4
thread pools and the default pool, exact stdout/JSON bytes, output inside the
source, and public error variants/paths. Unix checks additionally cover root and
interior links, broken links, invalid UTF-8 names/targets, permissions and special
entries. Capability-dependent tests report `SKIP` to stderr when unavailable;
use `cargo test --locked -- --nocapture` to expose these messages. Unix-only
encoding and permission semantics are not asserted on Windows. Windows root-link
checks run when symlink creation is available. Actual Windows
1 GiB read-boundary measurements require the opt-in large group.

`tests/regression_script.rs` supplies a fake Hyperfine that executes the serialized
commands and checks their argument boundaries. It exercises matching output,
differing output, differing trailing newlines, failure exits and shell-sensitive
paths without collecting misleading timing data. It runs on Unix with the normal
test suite; no Hyperfine installation is required for this check.

## Current-library benchmarks

The repaired Criterion benches use the current `paq` library. File-size cases
assert actual fixture lengths and method equivalence before timing; empty files
exercise the production shortcut without attempting an empty mapping. Sort input
is cloned before **each** timed operation using `BatchSize::PerIteration`, so
cloning/validation are outside the sort measurement and large clones cannot
accumulate. Distributions are explicitly random, sorted, reverse and duplicate
heavy. Finalization cases verify identical hashes against the existing
allocation/copy reference, including zero entries and duplicate digests.

`hash_source` measurements explicitly initialize the global Rayon pool before
sampling, and therefore describe warmed library calls. Hyperfine describes fresh
processes and includes their startup. Keep the two scopes separate in reports.

| Benchmark | Ordinary cases | Opt-in cases (`PAQ_BENCH_LARGE=1`) |
| --- | --- | --- |
| `hash_by_file_size` | 0/1, BLAKE3 block, 1,024-byte, platform buffer and 1 MiB boundaries | 64/256 MiB; Windows 1 GiB boundary |
| `hash_list_sort` | 0/1/100/1,000/2,000/5,000 digests; four distributions | 100,000/1,000,000 digests |
| `hash_list_hash` | 0/1/10/1,000 sorted digests | 100,000/1,000,000 digests |
| `functional` | Empty, single, 98/99/100/101/199/200/1,000 tiny files, mixed 128 files, deep/wide trees; both hidden modes | 8 × 8 MiB clustered files, 10,000 tiny files |

Generated large file bytes are deterministic and repeat an 8 KiB LCG block. This
is compressible data; do not call these fixtures incompressible or extrapolate
their storage behavior to arbitrary data. Fixture directories use atomic unique
creation and are cleaned up with the default `test-cleanup` feature. An existing
`TMPDIR` can place fixtures on a chosen filesystem. Setup occurs outside timing.

Run lightweight execution checks without collecting Criterion timing:

```bash
cargo bench --locked --bench hash_by_file_size -- --test
cargo bench --locked --bench hash_list_sort -- --test
cargo bench --locked --bench hash_list_hash -- --test
cargo bench --locked --bench functional -- --test
```

Select measurements with Criterion filters, for example:

```bash
RAYON_NUM_THREADS=4 cargo bench --locked --bench functional -- 'hash_source/tiny_1000_files'
PAQ_BENCH_LARGE=1 cargo bench --locked --bench hash_list_sort -- '/random/1000000'
PAQ_BENCH_LARGE=1 cargo bench --locked --bench hash_list_hash -- '/1000000'
PAQ_BENCH_GO=/data/go1.25.0 cargo bench --locked --bench functional -- 'hash_source/go1.25.0'
```

The optional Go path must contain the pinned corpus documented above, with `.git`
removed. Its provenance is the caller's responsibility. The historical
`hash_using_walkdir` and `hash_using_jwalk` benches retain v1.4.0 implementations;
they are historical comparisons, not evidence isolating traversal performance
in the current library. `comparison.sh` hashes a different set/format from paq
and cannot serve as its compatibility reference.

For each performance report, retain raw samples and record reference/candidate
revisions, lockfile, compiler/target/build flags, CPU model and core count, thread
environment, OS/kernel, filesystem/storage, fixture counts/sizes/content pattern,
hidden mode, cache state and unrelated system load. Report fresh-process runtime,
CPU user/system time, aggregate CPU utilization and peak resident memory
separately from warmed-library measurements. Use repeated, interleaved baseline
and candidate samples on immutable fixtures; label warm filesystem data honestly.
Do not claim cold-cache/storage-bound results without a controlled experiment.
Set regression budgets from baseline repeatability and workload priorities before
accepting candidates. Criterion stage speedups alone do not establish an
end-to-end speedup.
