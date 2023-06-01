//! Hash source on filesystem.
//!
//! For help:
//! ```bash
//! cargo run -- -h
//! ```

use clap::{
    crate_name,
    crate_description,
    crate_version,
    Command,
    Arg,
    ArgAction
};
use paq::hash_source;

fn main() {
    // todo - add error handling with messaging

    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .allow_external_subcommands(false)
        .arg(
            Arg::new("src")
                .help("Source to hash (filesystem path)")
                .default_value(".")
        )
        .arg(
            Arg::new("ignore-hidden")
                .short('i')
                .long("ignore-hidden")
                .help("Ignore files or directories starting with dot or full stop")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    let source = matches.get_one::<String>("src").unwrap();
    let ignore_hidden = matches.get_flag("ignore-hidden");
    let hash = hash_source(source, ignore_hidden);
    println!("{}", hash);
}