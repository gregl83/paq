use std::{
    cmp,
    env,
    error,
    io::Write,
    fs,
    path::{
        Path,
        PathBuf,
    },
    result,
};


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
/// We use this in lieu of tempfile because tempfile brings in too many dependencies.
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

    /// Create a file in temporary directory using data of byte array.
    pub fn new_file(&self, name: &str, data: &[u8]) -> Result<()> {
        let file_path = PathBuf::from(format!("{}/{}", self.path().display(), name));
        fs::write(file_path.as_os_str(), data)
            .map_err(|e| err!("failed to write data to {}: {}", file_path.display(), e))?;
        Ok(())
    }

    /// Create a file filled with pseudo-random non-zero bytes.
    ///
    /// This is critical for benchmarking compression or I/O, as it forces
    /// actual disk writes and prevents Run-Length Encoding (RLE) optimizations.
    pub fn new_file_with_random_data(&self, name: &str, file_size: u64) -> Result<()> {
        let file_path = PathBuf::from(format!("{}/{}", self.path().display(), name));

        const BUFFER_SIZE: usize = 8 * 1024; // reduce syscalls; 8KB is an efficient page size
        let mut buffer = [0u8; BUFFER_SIZE];

        // fill buffer with pseudo-random noise (avoiding 'rand' dependency)
        let mut state: u32 = 0xDEADBEEF;
        buffer.iter_mut().for_each(|byte| {
            // Linear Congruential Generator (LCG) step; high entropy
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *byte = (state >> 24) as u8;
        });

        let mut file = fs::File::create(&file_path)
            .map_err(|e| err!("failed to create file {}: {}", file_path.display(), e))?;

        let mut remaining: usize = file_size.try_into().unwrap();
        while remaining > 0 {
            // reuse random buffer to save CPU time (repeat every 8KB); otherwise, move within loop
            let to_write = cmp::min(remaining, BUFFER_SIZE);
            file.write_all(&buffer[..to_write])
                .map_err(|e| err!("failed to write data to {}: {}", file_path.display(), e))?;
            remaining -= to_write;
        }

        Ok(())
    }

    /// Return the underlying path to this temporary directory.
    pub fn path(&self) -> &Path {
        &self.0
    }
}
