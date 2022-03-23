[![Build Status](https://github.com/gregl83/paq/workflows/CI/badge.svg?branch=main)](https://github.com/gregl83/paq/actions?query=workflow%3ACI+branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/paq.svg)](https://crates.io/crates/paq)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/gregl83/paq/blob/master/LICENSE)

# paq

paq files to hash.

Hash file or directory (recursively).

Directories output the `top hash`, or `root`, of a [merkle tree](https://en.wikipedia.org/wiki/Merkle_tree).

Version Control System agnostic.

Powered by `SHA256` hashing algorithm.

## Install Command

Requires [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Run `cargo install paq`.

### Usage

Run `paq [src]` to hash source file or directory. 

For help, run `paq --help`.

## Use Crate

Add `paq` to project [dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-cratesio) in `Cargo.toml`.

### Usage

```rust
use paq;

let source = "/path/to/source";
let ignore_hidden = true; // .dir or .file
let source_hash: String = paq::hash_source(source, ignore_hidden);

println!("{}", source_hash);
```

## Content Limitations

Hashes are generated using file system content as input data to the `SHA256` hashing algorithm.

By design, `paq` does NOT include file system metadata in hash input such as:

- File modes
- File ownership
- File modification and access times
- File ACLs and extended attributes
- Hard links
- Symlink target contents

Additionally, files or directory contents starting with dot or full stop can be ignored.

## Example

The `./example` directory contains some sample files, subdirectory and a symlink to test `paq` functionality.

```rust
let source = "example";
let ignore_hidden = true;
let source_hash: String = paq::hash_source(source, ignore_hidden);

assert_eq!(source_hash, "2a13feb1fd6f81de8229de8f676e854c151b091e5e04f2c4d27bcde4e448623b");
```

Note: expect different results if `ignore_hidden` is set to `false`.

## License

[MIT](LICENSE)
