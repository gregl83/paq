use std::env;
use std::error;
use std::fs::{self};
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
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
pub struct TempDir(PathBuf);

#[cfg(feature = "test-cleanup")]
impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

impl TempDir {
    /// Create a new empty temporary directory under the system's configured
    /// temporary directory.
    pub fn new(name: &str) -> Result<TempDir> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = env::temp_dir().join(TEMP_DIRECTORY_NAME);
        fs::create_dir_all(&root)?;
        for _ in 0..100 {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!("{}-{id}-{name}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(TempDir(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Err(err!("failed to create a unique temporary directory"))
    }

    /// Create a new file in temporary directory using data of byte array.
    pub fn new_file(&self, name: &str, data: &[u8]) -> Result<()> {
        let file_path = self.path().join(name);
        fs::write(file_path.as_os_str(), data).expect("Unable to write file");
        Ok(())
    }

    /// Read a file in temporary directory.
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        let file_path = self.path().join(name);
        Ok(fs::read(file_path.as_os_str()).expect("Unable to read file"))
    }

    /// Create a new symlink in temporary directory to target.
    #[cfg(target_family = "unix")]
    pub fn new_symlink(&self, name: &str, target: PathBuf) -> Result<()> {
        let symlink_path = self.path().join(name);
        symlink(target.as_os_str(), symlink_path.as_os_str()).expect("Unable to create symlink");
        Ok(())
    }

    /// Return the underlying path to this temporary directory.
    pub fn path(&self) -> &Path {
        &self.0
    }
}
