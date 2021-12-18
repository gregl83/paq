# paq

paq files to hash.

Hash a single file or all files in directory recursively.

## Installation

Requires [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Run `cargo install paq`.

### Usage

Run `paq [src]` to hash src (file or directory). 

For help, run `paq --help`.

## Library

Add `paq: "0.4.0"` to `Cargo.toml`.

### Usage

```rust
use paq;

let source = "/path/to/source";
let hash: String = paq::hash_source(source);

println!("{}", hash);
```

## Caution

Needs tests!

## License

[MIT](LICENSE)
