[![Build Status](https://github.com/gregl83/paq/workflows/CI/badge.svg?branch=main)](https://github.com/gregl83/paq/actions?query=workflow%3ACI+branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/paq.svg)](https://crates.io/crates/paq)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/gregl83/paq/blob/master/LICENSE)

# paq

Hash file or directory recursively.

Powered by `blake3` cryptographic hashing algorithm.

## Usage

Install the command line interface executable or use the crate library.

Included in this repository is an [example directory](./example) containing some sample files, a subdirectory and a symlink to test `paq` functionality.

### Executable

Installation requires [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Run `cargo install paq`.

#### Invoke Command

Run `paq [src]` to hash source file or directory. 

For help, run `paq --help`.

#### Hash Example Directory

```paq ./example```

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

assert_eq!(&source_hash[..], "494f366c528a930bb654b58721ab01683146381e1d2bf3e187311f9b725bfa19");
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
- Symlink target contents

Additionally, files or directory contents starting with dot or full stop *can* optionally be ignored.

## How it Works

1. Recursively get path(s) for a given source argument.
2. Hash each path and file content if path is for a file.
3. Sort the list of hashes to maintain consistent ordering.
4. Compute the final hash by hashing the sorted list of hashes.

## License

[MIT](LICENSE)
