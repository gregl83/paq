[![CI](https://github.com/gregl83/paq/actions/workflows/ci.yml/badge.svg)](https://github.com/gregl83/paq/actions/workflows/ci.yml)
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

The environment can be reproduced by creating a new CloudFormation stack using the [ec2-benchmark-template.yaml](../infra/ec2-benchmark-template.yaml) template.

The benchmark instance defaults to `c6a.4xlarge` (16 vCPUs, 32 GiB RAM).

### Git Data Source Snapshot

The [Go](https://github.com/golang/go) programming language repository serves as the data source for directory hashing benchmarks.

To ensure consistency, the benchmarks target the specific version tag [go1.25.0](https://github.com/golang/go/releases/tag/go1.25.0) (commit hash: `6e676ab2b809d46623acb5988248d95d1eb7939c`). Clone and checkout steps are defined in the [ec2-benchmark-template.yaml](../infra/ec2-benchmark-template.yaml) template.

> **Note:** The data source repository's `.git` directory is deleted prior to execution. This eliminates variability caused by version control metadata.

### Nix Package Manager

The [Nix](https://search.nixos.org/packages) package manager is used to pin software builds and third-party tools to specific, hash-verified versions.

The benchmark compute instance relies on the `paq` [flake.nix](../flake.nix) configuration to set up the environment. Tool sources, build toolchains, and dependencies are pinned through the Nix configuration, [flake.lock](../flake.lock), and package dependency hashes or lockfiles. The shell prints the tool versions on entry. All compilation and dependency installation happen before the timed commands.

### Hyperfine

Benchmarks are executed using [hyperfine](https://github.com/sharkdp/hyperfine).

The [bin](../bin) directory contains a helper script, [comparison.sh](../bin/comparison.sh), which invokes `hyperfine` to run comparative benchmarks against other tools.

From the paq checkout, run the comparison against the EC2 template's Go corpus:

```bash
nix develop .#benchmark
bash bin/verify-checksum-pipelines.sh /mnt/benchmark/target
bash bin/comparison.sh /mnt/benchmark/target
```

The runner performs three warmup runs and 20 measured runs per command,
including when invoked by the EC2 template. An explicit `--runs N` or `-r N`
overrides the measured run count. Additional Hyperfine options are passed through.

The comparison measures default CLI performance with a warm filesystem cache.
The runner does not clear caches between runs.

Before publishing timings, run [verify-checksum-pipelines.sh](../bin/verify-checksum-pipelines.sh)
on the benchmark VM. It checks that fd selects exactly the same files as find,
then verifies each parallel pipeline against a serial reference over three runs.
The serial reference sorts checksum records too; equality with the old aggregate
digest is not expected. fd 10.1.0 is supplied by the same hash-pinned benchmark Nixpkgs
snapshot as the other command-line utilities, and its version is printed on entry.

### Compared tools

The following commands match [comparison.sh](../bin/comparison.sh), with `./go`
as the example target directory. Versions correspond to the benchmark shell.

| Tool                                                                | Version | Command                                                                        | Algorithm |
| :------------------------------------------------------------------ | :------ | :----------------------------------------------------------------------------- | :-------- |
| [b3sum](https://github.com/BLAKE3-team/BLAKE3/tree/master/b3sum)    | 1.5.1   | `fd -HI -tf . ./go -0 \| xargs -0 -P0 b3sum \| LC_ALL=C sort \| b3sum`         | BLAKE3    |
| [checksumdir](https://pypi.org/project/checksumdir/)                | 1.3.0   | `checksumdir -a sha256 ./go`                                                   | SHA-256   |
| [directory-checksum](https://github.com/MShekow/directory-checksum) | 1.4.20  | `directory-checksum --max-depth=0 ./go`                                        | SHA-1     |
| [dirhash](https://github.com/andhus/dirhash-python)                 | 0.5.0   | `dirhash ./go -a sha256`                                                       | SHA-256   |
| [folder-hash](https://github.com/marc136/node-folder-hash)          | 4.1.1   | `folder-hash ./go`                                                             | SHA-1     |
| [GNU md5sum](https://www.gnu.org/software/coreutils/)               | 9.11    | `fd -HI -tf . ./go -0 \| xargs -0 -P0 md5sum \| LC_ALL=C sort \| md5sum`       | MD5       |
| [GNU sha2](https://www.gnu.org/software/coreutils/)                 | 9.11    | `fd -HI -tf . ./go -0 \| xargs -0 -P0 sha256sum \| LC_ALL=C sort \| sha256sum` | SHA-256   |
| [Hashrat](https://github.com/ColumPaget/Hashrat)                    | 1.25    | `hashrat -sha256 -dir -hidden ./go`                                            | SHA-256   |
| [merkle_hash](https://github.com/hristogochev/merkle_hash)          | 3.9.0   | `merkle-hash ./go`                                                             | BLAKE3    |
| [paq](https://github.com/gregl83/paq)                               | 2.0.0   | `paq ./go`                                                                     | BLAKE3    |

All commands include hidden entries. Their handling of names, empty directories,
and symbolic links differs:

| Tool                | Names in hash input                                                 | Empty directories | Symbolic links                                             |
| :------------------ | :------------------------------------------------------------------ | :---------------- | :--------------------------------------------------------- |
| b3sum pipeline      | File paths in checksum output, including the supplied target prefix | Omitted           | Omitted by `fd -tf`                                        |
| checksumdir         | Omitted; hashes file contents only                                  | Omitted           | Reads file-link targets; does not traverse directory links |
| directory-checksum  | Child names in directory listings                                   | Included          | Hashes target-path text                                    |
| dirhash             | File and directory names                                            | Omitted           | Follows file and directory links                           |
| folder-hash         | File and directory names, including the root name                   | Included          | Follows file and directory links                           |
| GNU md5sum pipeline | File paths in checksum output, including the supplied target prefix | Omitted           | Omitted by `fd -tf`                                        |
| GNU sha2 pipeline   | File paths in checksum output, including the supplied target prefix | Omitted           | Omitted by `fd -tf`                                        |
| Hashrat             | Omitted; hashes concatenated file contents in traversal order       | Omitted           | Reads file-link targets; does not traverse directory links |
| merkle_hash         | File and directory names, including the root name                   | Included          | Follows file and directory links                           |
| paq                 | Relative entry paths                                                | Included          | Hashes target-path text                                    |

These runs compare execution time; the tools use different hash formats and are
not expected to produce identical hashes. Use a tree without symbolic links,
such as the pinned Go corpus, and keep hidden files included. Keep the target
path spelling consistent because some tools include it or the root name in the hash.

The commands use default processing settings unless shown otherwise. paq and
merkle_hash use parallel processing; dirhash uses its default single worker.
The `fd -HI -tf` pipelines discover files in parallel, including hidden and
ignored files. `xargs -0 -P0` hashes with default batching and tool settings; no
process counts, batch sizes, or BLAKE3 thread counts are tuned. Ordinary
`LC_ALL=C sort` orders the newline-delimited checksum records after hashing,
then the final checksum command hashes that listing. `sort -z` is not used for
checksum output. These aggregate digests can differ from the previous pipelines,
which sorted paths before hashing.

directory-checksum's `--max-depth=0` prints only the root
checksum while still traversing and hashing the full tree. folder-hash prints
the hash tree using its default CLI output. Output generation is part of the
timed commands. MD5 is included as a legacy performance baseline.

Hashrat uses its published CLI with `-dir` to recursively hash the directory into
one digest and `-hidden` to include dotfiles and hidden directories. The pinned
build offers neither BLAKE3 nor BLAKE2, so `-sha256` matches the SHA-256 choice
used for the other SHA-2 comparisons. Directory mode hashes the concatenated
file contents in traversal order, without names or file boundaries; renaming
files can still change the digest by changing that order.

#### merkle_hash wrapper

`merkle-hash` is a small [benchmark wrapper](../benches/merkle_hash) that selects
BLAKE3, enables filename hashing, and prints the directory root hash. It retains
the library's default parallel processing. Its dependencies are pinned in a
separate Cargo lockfile, and its release profile matches paq's.

To build and run the wrapper without Nix:

```bash
cargo build --release --locked --manifest-path benches/merkle_hash/Cargo.toml --target-dir target
./target/release/merkle-hash /mnt/benchmark/target
```

## Regression Testing

Benchmarks are used to ensure that `paq` release candidates have equal or better performance than past releases.
