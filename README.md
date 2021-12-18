# paq

paq files to hash.

Hash a single file or all files in directory recursively.

## Installation

1. Clone repository.
2. Run `cargo install --path .` from root of repository.

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
