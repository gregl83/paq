//! Hash source on filesystem.
//!
//! For help:
//! ```bash
//! cargo run -- -h
//! ```

use clap::{
    builder::TypedValueParser, crate_description, crate_name, crate_version, error::ContextKind,
    error::ContextValue, error::ErrorKind, Arg, ArgAction, Command,
};
use paq::hash_source;
use std::{
    fs::File,
    io::{
        Error,
        Write,
    },
    path::{
        Path,
        PathBuf,
    }
};

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct PathBufferValueParser {
    validate_exists: bool,
}

impl TypedValueParser for PathBufferValueParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = PathBuf::from(value);
        if self.validate_exists && !path.exists() {
            let mut err = clap::Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.unwrap().to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value.to_string_lossy().into_owned()),
            );
            err.insert(
                ContextKind::ValidValue,
                ContextValue::Strings(vec![String::from("valid file or directory path")]),
            );
            return Err(err);
        }
        Ok(path)
    }
}

fn derive_output_filepath(source: &Path) -> PathBuf {
    let source_canonical = source.canonicalize().unwrap();
    let source_filename = source_canonical.file_name().unwrap().to_str().unwrap();
    let mut path_buffer = PathBuf::from(source_canonical.parent().unwrap());
    path_buffer.push(format!("{source_filename}.paq"));
    path_buffer
}

fn write_hashfile(filepath: &PathBuf, hash: &str) -> Result<(), Error> {
    let mut file = File::create(filepath).unwrap();
    file.write_all(format!("\"{hash}\"").as_bytes())
}

fn main() {
    let output_default = "<src>.paq";
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .allow_external_subcommands(false)
        .arg(
            Arg::new("src")
                .value_parser(PathBufferValueParser {
                    validate_exists: true,
                })
                .default_value(".")
                .help("Source to hash (filesystem path)"),
        )
        .arg(
            Arg::new("ignore-hidden")
                .short('i')
                .long("ignore-hidden")
                .action(ArgAction::SetTrue)
                .help("Ignore files or directories starting with dot or full stop"),
        )
        .arg(
            Arg::new("filepath")
                .short('o')
                .long("out")
                .value_parser(PathBufferValueParser {
                    validate_exists: false,
                })
                .require_equals(true)
                .num_args(0..=1)
                .default_missing_value(output_default)
                .help(format!(
                    "Output hash (filesystem path) [default: {output_default}]"
                )),
        )
        .after_help("Fails if operating system denies read access to any source file.")
        .get_matches();

    let source = matches.get_one::<PathBuf>("src").unwrap();
    let ignore_hidden = matches.get_flag("ignore-hidden");
    let output: Option<&PathBuf> = matches.get_one::<PathBuf>("filepath");
    let hash = hash_source(source, ignore_hidden);

    if let Some(filepath) = output {
        let output_filepath = match filepath.to_str().unwrap() {
            s if s == output_default => derive_output_filepath(source),
            _ => filepath.to_path_buf(),
        };
        write_hashfile(&output_filepath, hash.as_str()).unwrap();
    }

    println!("{hash}");
}
