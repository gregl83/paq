[![Build](https://github.com/gregl83/paq/actions/workflows/build.yml/badge.svg)](https://github.com/gregl83/paq/actions/workflows/build.yml)
[![Coverage Status](https://codecov.io/gh/gregl83/paq/graph/badge.svg?token=CL93O7DW9C)](https://codecov.io/gh/gregl83/paq)
[![Crates.io](https://img.shields.io/crates/v/paq.svg)](https://crates.io/crates/paq)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/gregl83/paq/blob/master/LICENSE)

# [paq](/) / Benchmarks

This document outlines the process for creating reproducible and comparative benchmarks for `paq`.

Benchmarks are located in the [benches](../benches) directory of this repository.

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

The [benches](../benches) directory contains a helper script, [hyperfine.sh](../benches/hyperfine.sh), which invokes `hyperfine` to run comparative benchmarks against other tools.

Hyperfine benchmark commands starting with `find` use the following command with various `<hashsum>` implementations:

```bash
find ./go -type f -print0 | LC_ALL=C sort -z | xargs -0 <hashsum> | <hashsum>
```

## Regression Testing

Benchmarks are used to ensure that `paq` release candidates have equal or better performance to the prior latest release.