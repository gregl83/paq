#[allow(dead_code)]
mod utils;

#[path = "lib/hashing.rs"]
mod hashing;

#[path = "lib/filtering.rs"]
mod filtering;

#[cfg(target_family = "unix")]
#[path = "lib/symlinks.rs"]
mod symlinks;

#[path = "lib/collisions.rs"]
mod collisions;

#[path = "lib/errors.rs"]
mod errors;
