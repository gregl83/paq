use std::env;
use std::error;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::result;

pub const TEMP_DIRECTORY_NAME: &str = "paq";

/// A convenient result type alias.
pub type Result<T> = result::Result<T, Box<dyn error::Error + Send + Sync>>;

/// Create an error from a format!-like syntax.
#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => {
        Box::<dyn error::Error + Send + Sync>::from(format!($($tt)*))
    }
}

/// A simple wrapper for creating a temporary directory that is automatically
/// deleted when it's dropped.
///
/// We use this in lieu of tempfile because tempfile brings in too many
/// dependencies.
#[derive(Debug)]
pub struct TempDir(PathBuf, PathBuf);

#[cfg(feature = "test-cleanup")]
impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.1).unwrap();
    }
}

impl TempDir {
    /// Create a new empty temporary directory under the system's configured
    /// temporary directory.
    pub fn new(name: &str) -> Result<TempDir> {
        static TRIES: usize = 100;

        let tmpdir = env::temp_dir();
        for _ in 0..TRIES {
            let root_path = tmpdir.join(TEMP_DIRECTORY_NAME);
            let iteration_path = root_path.join(name);
            if iteration_path.is_dir() {
                continue;
            }
            fs::create_dir_all(&iteration_path).map_err(|e| {
                err!("failed to create {}: {}", iteration_path.display(), e)
            })?;
            return Ok(TempDir(root_path, iteration_path));
        }
        Err(err!("failed to create temp dir after {} tries", TRIES))
    }

    /// Create a new file in temporary directory using data of byte array.
    pub fn new_file(&self, name: &str, data: &[u8]) -> Result<()> {
        let file_path = PathBuf::from(format!("{}/{}", self.path().display(), name));
        Ok(fs::write(file_path.as_os_str(), data).expect("Unable to write file"))
    }

    /// Return the underlying path to this temporary directory.
    pub fn path(&self) -> &Path {
        &self.1
    }
}