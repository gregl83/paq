//! Hash source on filesystem.
//!
//! For help:
//! ```bash
//! cargo run -- -h
//! ```

use std::path::PathBuf;
use clap::{
    crate_name,
    crate_description,
    crate_version,
    builder::TypedValueParser,
    error::ErrorKind,
    error::ContextKind,
    error::ContextValue,
    Command,
    Arg,
    ArgAction
};
use paq::hash_source;

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct PathBufferValueParser {}

impl TypedValueParser for PathBufferValueParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = PathBuf::from(value);
        if !path.exists() {
            let mut err = clap::Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.unwrap().to_string())
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value.to_string_lossy().into_owned())
            );
            err.insert(
                ContextKind::ValidValue,
                ContextValue::Strings(vec![
                    String::from("valid file or directory path")
                ])
            );
            return Err(err);
        }
        Ok(path)
    }
}

fn main() {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .allow_external_subcommands(false)
        .arg(
            Arg::new("src")
                .help("Source to hash (filesystem path)")
                .value_parser(PathBufferValueParser{})
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

    let source = matches.get_one::<PathBuf>("src").unwrap();
    let ignore_hidden = matches.get_flag("ignore-hidden");
    let hash = hash_source(&source, ignore_hidden);
    println!("{}", hash);
}