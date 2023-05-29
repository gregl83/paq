[![Build Status](https://github.com/gregl83/paq/workflows/CI/badge.svg?branch=main)](https://github.com/gregl83/paq/actions?query=workflow%3ACI+branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/paq.svg)](https://crates.io/crates/paq)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/gregl83/paq/blob/master/LICENSE)

# paq

paq files to hash.

Hash file or directory (recursively).

Powered by `blake3` cryptographic hashing algorithm.

## Install Command

Requires [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Run `cargo install paq`.

### Usage

Run `paq [src]` to hash source file or directory. 

For help, run `paq --help`.

## Use Crate Library

Add `paq` to project [dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-cratesio) in `Cargo.toml`.

### Usage

```rust
use paq;

let source = "/path/to/source";
let ignore_hidden = true; // .dir or .file
let source_hash: paq::ArrayString<64> = paq::hash_source(source, ignore_hidden);

println!("{}", source_hash);
```

## Content Limitations

Hashes are generated using file system content as input data to the `blake3` hashing algorithm.

By design, `paq` does NOT include file system metadata in hash input such as:

- File modes
- File ownership
- File modification and access times
- File ACLs and extended attributes
- Hard links
- Symlink target contents

Additionally, files or directory contents starting with dot or full stop *can* optionally be ignored.

## Example

The `./example` directory contains some sample files, subdirectory and a symlink to test `paq` functionality.

```rust
use paq;

let source = "example";
let ignore_hidden = true;
let source_hash: paq::ArrayString<64> = paq::hash_source(source, ignore_hidden);

assert_eq!(&source_hash[..], "778c013fbdb4d129357ec8023ea1d147e60a014858cfc2dd998af6c946e802a9");
```

Expect different results if `ignore_hidden` is set to `false`.

## How it Works

1. Recursively get all files for a given source argument.
2. Hash each file using the file's relative path and content as input to the hash function.
3. Sort the list of file hashes.
4. Calculate the final hash using the file hashes concatenated as input to the hash function.

## License

[MIT](LICENSE)
