#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
use std::{
    env,
    error,
    fs::{
        self,
    },
    path::{
        Path,
        PathBuf,
    },
    result,
};

pub const TEMP_DIRECTORY_NAME: &str = "paq";

/// A convenient result type alias.
pub type Result<T> = result::Result<T, Box<dyn error::Error + Send + Sync>>;

/// Assert that two trees have distinct hashes with either hidden-file setting.
pub fn assert_distinct_tree_hashes(left: &TempDir, right: &TempDir) {
    for ignore_hidden in [false, true] {
        assert_ne!(
            paq::try_hash_source(left.path(), ignore_hidden).unwrap(),
            paq::try_hash_source(right.path(), ignore_hidden).unwrap(),
            "distinct trees must not collide (ignore_hidden={ignore_hidden})"
        );
    }
}

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
        static TRIES: usize = 100;

        let tmpdir = env::temp_dir();
        for _ in 0..TRIES {
            let root_path = tmpdir.join(TEMP_DIRECTORY_NAME);
            let iteration_path = root_path.join(name);
            if iteration_path.is_dir() {
                continue;
            }
            fs::create_dir_all(&iteration_path)
                .map_err(|e| err!("failed to create {}: {}", iteration_path.display(), e))?;
            return Ok(TempDir(iteration_path));
        }
        Err(err!("failed to create temp dir after {} tries", TRIES))
    }

    /// Create a new file in temporary directory using data of byte array.
    pub fn new_file(&self, name: &str, data: &[u8]) -> Result<()> {
        let file_path = PathBuf::from(format!("{}/{}", self.path().display(), name));
        fs::write(file_path.as_os_str(), data).expect("Unable to write file");
        Ok(())
    }

    /// Read a file in temporary directory.
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        let file_path = PathBuf::from(format!("{}/{}", self.path().display(), name));
        Ok(fs::read(file_path.as_os_str()).expect("Unable to read file"))
    }

    /// Create a new symlink in temporary directory to target.
    #[cfg(target_family = "unix")]
    pub fn new_symlink(&self, name: &str, target: PathBuf) -> Result<()> {
        let symlink_path = PathBuf::from(format!("{}/{}", self.path().display(), name));
        symlink(target.as_os_str(), symlink_path.as_os_str()).expect("Unable to create symlink");
        Ok(())
    }

    /// Return the underlying path to this temporary directory.
    pub fn path(&self) -> &Path {
        &self.0
    }
}
