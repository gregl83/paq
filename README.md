[![Build](https://github.com/gregl83/paq/actions/workflows/build.yml/badge.svg)](https://github.com/gregl83/paq/actions/workflows/build.yml)
[![Coverage Status](https://codecov.io/gh/gregl83/paq/graph/badge.svg?token=CL93O7DW9C)](https://codecov.io/gh/gregl83/paq)
[![Crates.io](https://img.shields.io/crates/v/paq.svg)](https://crates.io/crates/paq)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/gregl83/paq/blob/master/LICENSE)

# paq

Hash file or directory recursively.

Powered by `blake3` cryptographic hashing algorithm.

<p align="center">
  <img src="paq.gif" alt="paq hashing demo" />
</p>

## Performance

The [Go](https://github.com/golang/go/commit/6e676ab2b809d46623acb5988248d95d1eb7939c) programming language repository was used as a test data source (157 MB / 14,490 files).

| Tool                       | Version | Command                   |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------------------- | :------ | :------------------------ | ------------: | -------: | -------: | -----------: |
| [paq][paq]                 | latest  | `paq ./go`                |    77.7 ± 0.6 |     77.1 |     80.2 |         1.00 |
| [b3sum][b3sum]             | 1.5.1   | `find ./go ... b3sum`     |   327.3 ± 3.6 |    320.2 |    332.3 |  4.21 ± 0.05 |
| [dirhash][dirhash]         | 0.5.0   | `dirhash -a sha256 ./go`  |   576.1 ± 2.9 |    570.8 |    580.3 |  7.41 ± 0.06 |
| [GNU sha2][gnusha]         | 9.7     | `find ./go ... sha256sum` |  725.2 ± 43.5 |    692.2 |    813.2 |  9.33 ± 0.56 |
| [folder-hash][folder-hash] | 4.1.1   | `folder-hash ./go`        | 1906.0 ± 78.0 |   1810.0 |   2029.0 | 24.53 ± 1.02 |

[paq]: https://github.com/gregl83/paq
[b3sum]: https://github.com/BLAKE3-team/BLAKE3/tree/master/b3sum
[gnusha]: https://manpages.debian.org/testing/coreutils/sha256sum.1.en.html
[dirhash]: https://github.com/andhus/dirhash-python
[folder-hash]: https://github.com/marc136/node-folder-hash

See [benchmarks](docs/benchmarks.md) documentation for more details.

## Installation

### Pre-Built Binary

Windows, macOS, and Ubuntu are supported.

1. **Download:** Go to the [Latest Release](https://github.com/gregl83/paq/releases) page and download the `.zip` archive matching your OS and Architecture.
2. **Extract:** Unzip the `.zip` archive to retrieve the `paq` binary.
3. **Install:** Make the `paq` binary executable (e.g., `chmod +x`) and move it to a directory in your system PATH.
4. **Verify:** Confirm installation by running `paq --version` from the Command Line Interface.

### Cargo Install

Requires the [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) package manager.

#### Install From Crates.io

```bash
cargo install paq
```

#### Install From Repository Clone (Unstable)

Not recommended due to instability of `main` branch in-between tagged releases.

1. Clone this repository.
2. Run `cargo install --path .` from repository root.

### Nix Flakes

Requires [nix](https://nix.dev/) and the `nix-command` [experimental feature](https://nixos.wiki/wiki/Flakes#Enable_flakes_temporarily) to be enabled.

```bash
nix profile add github:gregl83/paq
```

### Python Package

Support for Python is available in the [paqpy](https://github.com/gregl83/paqpy) package.

## Usage

Command Line Interface executable or Crate library.

Included in this repository is an [example directory](./example) containing some sample files, a subdirectory and a symlink to test `paq` functionality.

### Executable

Run `paq [src]` to hash source file or directory.

Output hash to `.paq` file as valid JSON.

For help, run `paq --help`.

#### Hash Example Directory

```bash
paq ./example
```

Path to example directory can be relative or absolute.

Expect different results if `-i` or `--ignore-hidden` flag argument is used.

### Crate Library

Add `paq` to project [dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-cratesio) in `Cargo.toml`.

#### Use Library

```rust
use paq;

let source = std::path::PathBuf::from("/path/to/source");
let ignore_hidden = true; // .dir or .file
let source_hash: paq::ArrayString<64> = paq::hash_source(&source, ignore_hidden);

println!("{}", source_hash);
```

#### Hash Example Directory

```rust
use paq;

let source = std::path::PathBuf::from("example");
let ignore_hidden = true;
let source_hash: paq::ArrayString<64> = paq::hash_source(&source, ignore_hidden);

assert_eq!(&source_hash[..], "d7d25c9b2fdb7391e650085a985ad0d892c7f0dd5edd32c7ccdb4b0d1c34c430");
```

Expect different results if `ignore_hidden` is set to `false`.

## Content Limitations

Hashes are generated using file system content as input data to the `blake3` hashing algorithm.

By design, `paq` does NOT include file system metadata in hash input such as:

- File modes
- File ownership
- File modification and access times
- File ACLs and extended attributes
- Hard links
- Symlink target contents (target path is hashed)

Additionally, files or directory contents starting with dot or full stop _can_ optionally be ignored.

## How it Works

1. **Stream & Hash:** Recursively discovers source system path(s) and hashes them in a parallel pipeline.
2. **Sort:** Orders the hashes to ensure a deterministic output.
3. **Finalize:** Computes the final hash by hashing the list of hashes.

## License

[MIT](LICENSE)
