//! Hash source on filesystem.
//!
//! For help:
//! ```bash
//! cargo run -- -h
//! ```

use anyhow::Context;
use clap::{
    builder::TypedValueParser, crate_description, crate_name, crate_version, error::ContextKind,
    error::ContextValue, error::ErrorKind, Arg, ArgAction, Command,
};
use paq::try_hash_source;
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

fn derive_output_filepath(source: &Path) -> Result<PathBuf, Error> {
    let source_canonical = source.canonicalize()?;
    let mut source_filename = source_canonical
        .file_name()
        .ok_or_else(|| Error::other("source path has no file name"))?
        .to_os_string();
    source_filename.push(".paq");
    Ok(source_canonical.with_file_name(source_filename))
}

fn write_hashfile(filepath: &Path, hash: &str) -> Result<(), Error> {
    let mut file = File::create(filepath)?;
    file.write_all(format!("\"{hash}\"").as_bytes())
}

fn main() -> anyhow::Result<()> {
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
    let hash = try_hash_source(source, ignore_hidden)
        .with_context(|| format!("failed to hash `{}`", source.display()))?;

    if let Some(filepath) = output {
        let output_filepath = if filepath.as_path() == Path::new(output_default) {
            derive_output_filepath(source).with_context(|| {
                format!("failed to derive output path for `{}`", source.display())
            })?
        } else {
            filepath.to_path_buf()
        };
        write_hashfile(&output_filepath, hash.as_str()).with_context(|| {
            format!("failed to write hash to `{}`", output_filepath.display())
        })?;
    }

    println!("{hash}");
    Ok(())
}
