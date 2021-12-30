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
        .version("0.5.0")
        .about("paq files to hash.")
        .arg(Arg::with_name("src")
            .help("Source to hash (filesystem path)")
            .default_value(".")
            .index(1))
        .arg(Arg::with_name("ignore-hidden")
            .help("Ignore files or directories starting with dot or full stop")
            .long("ignore-hidden")
            .short("i")
        )
        .get_matches();

    let source = matches.value_of("src").unwrap();
    let ignore_hidden = matches.is_present("ignore-hidden");
    let hash = lib::hash_source(source, ignore_hidden);
    println!("{}", hash);
}
