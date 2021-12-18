//! Hash source on filesystem.
//!
//! For help:
//! ```bash
//! cargo run -- -h
//! ```

use clap::{App, Arg};

mod lib;

fn main() {
    // todo - add error handling with messaging

    let matches = App::new("paq")
        .version("0.3.2")
        .about("paq files to hash.")
        .arg(Arg::with_name("src")
            .help("Source to hash (path)")
            .default_value(".")
            .index(1))
        .get_matches();

    let source = matches.value_of("src").unwrap();
    let hash = lib::hash_source(source);
    println!("{}", hash);
}
