# paq

paq files to hash.

Hash file or directory (recursively).

Directories output the `top hash`, or `root`, of a [merkle tree](https://en.wikipedia.org/wiki/Merkle_tree).

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

## Caution

Needs tests!

## License

[MIT](LICENSE)
