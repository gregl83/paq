#[allow(dead_code)]
mod utils;

#[allow(deprecated)]
#[path = "bin/output.rs"]
mod output;

#[allow(deprecated)]
#[path = "bin/filtering.rs"]
mod filtering;

#[allow(deprecated)]
#[path = "bin/errors.rs"]
mod errors;

#[cfg(target_family = "unix")]
#[allow(deprecated)]
#[path = "bin/symlinks.rs"]
mod symlinks;
