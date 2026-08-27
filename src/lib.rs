use std::{
    fs,
    io::{self, prelude::*},
    iter,
    path::{Path, PathBuf},
};

pub use arrayvec::ArrayString;
use blake3::Hasher;
use memmap2::Mmap;
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

pub const PATH_BATCH_SIZE: usize = 100;
pub const MAX_FILE_SIZE_FOR_UNBUFFERED_READ: u64 = 1024 + 1;
#[cfg(not(target_os = "windows"))]
pub const MIN_FILE_SIZE_FOR_MMAP_READ: u64 = 1024 * 1024 - 1;
#[cfg(target_os = "windows")]
pub const MIN_FILE_SIZE_FOR_MMAP_READ: u64 = 1024 * 1024 * 1024 - 1;
#[cfg(not(target_os = "windows"))]
pub const FILE_BUFFER_SIZE: usize = 32 * 1024;
#[cfg(target_os = "windows")]
pub const FILE_BUFFER_SIZE: usize = 128 * 1024;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("failed to access path `{}`: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("path is not valid UTF-8: {}", .0.display())]
    InvalidUtf8Path(PathBuf),
    #[error(
        "path `{}` is outside source `{}`",
        path.display(),
        root.display()
    )]
    OutsideSource { path: PathBuf, root: PathBuf },
    #[error("failed to traverse source: {0}")]
    Walk(#[from] walkdir::Error),
}

#[inline]
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s != "." && s.starts_with("."))
        .unwrap_or(false)
}

#[inline]
fn filter(ignore_hidden: bool) -> impl FnMut(&DirEntry) -> bool {
    if ignore_hidden {
        |entry: &DirEntry| -> bool { !is_hidden(entry) }
    } else {
        |_: &DirEntry| -> bool { true }
    }
}

fn try_buffer_file_to_hasher(hasher: &mut Hasher, path: &Path) -> Result<(), Error> {
    let mut file = fs::File::open(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut buffer = [0; FILE_BUFFER_SIZE];
    loop {
        let buffer_size = file.read(&mut buffer[..]).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if buffer_size == 0 {
            break;
        }
        hasher.update(&buffer[..buffer_size]);
    }
    Ok(())
}

fn try_hash_path(root: &Path, entry: &DirEntry) -> Result<[u8; 32], Error> {
    let path = entry.path();
    let source_path = path
        .strip_prefix(root)
        .map_err(|_| Error::OutsideSource {
            path: path.to_path_buf(),
            root: root.to_path_buf(),
        })?
        .to_str()
        .ok_or_else(|| Error::InvalidUtf8Path(path.to_path_buf()))?;
    let source_type = entry.file_type();

    let mut hasher = Hasher::new();
    // hash paths for fs changes other than file content (must be relative to root)
    #[cfg(target_family = "unix")]
    {
        hasher.update(source_path.as_bytes());
    }
    #[cfg(target_family = "windows")]
    {
        hasher.update(source_path.replace("\\", "/").as_bytes());
    }
    if source_type.is_symlink() {
        // for symlinks add hash of target path
        let symlink_target_path = fs::read_link(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let symlink_target = symlink_target_path
            .to_str()
            .ok_or_else(|| Error::InvalidUtf8Path(symlink_target_path.clone()))?;
        #[cfg(target_family = "unix")]
        {
            hasher.update(symlink_target.as_bytes());
        }
        #[cfg(target_family = "windows")]
        {
            hasher.update(symlink_target.replace("\\", "/").as_bytes());
        }
    } else if source_type.is_file() {
        // for files, add contents to hasher
        let metadata = entry.metadata()?;
        let file_size = metadata.len();
        if file_size == 0 {
            // empty file, return immediately
            return Ok(*hasher.finalize().as_bytes());
        } else if file_size < MAX_FILE_SIZE_FOR_UNBUFFERED_READ {
            // small file read using unbuffered
            let file = fs::read(path).map_err(|source| Error::Io {
                path: path.to_path_buf(),
                source,
            })?;
            hasher.update(&file);
        } else if file_size > MIN_FILE_SIZE_FOR_MMAP_READ {
            // large size files read using mmap or fail to buffered read
            let file = fs::File::open(path).map_err(|source| Error::Io {
                path: path.to_path_buf(),
                source,
            })?;
            match unsafe { Mmap::map(&file) } {
                Ok(mmap) => {
                    hasher.update(&mmap);
                }
                Err(_) => {
                    try_buffer_file_to_hasher(&mut hasher, path)?;
                }
            }
        } else {
            // medium file size read using buffer
            try_buffer_file_to_hasher(&mut hasher, path)?;
        }
    }
    Ok(*hasher.finalize().as_bytes())
}

fn get_hashes_root(file_hashes: Vec<[u8; 32]>) -> ArrayString<64> {
    let mut flattened_bytes = Vec::with_capacity(file_hashes.len() * 32);

    for file_hash in &file_hashes {
        flattened_bytes.extend_from_slice(file_hash);
    }

    blake3::hash(&flattened_bytes).to_hex()
}

/// Hash file system source.
///
/// Source **must** be a path to a file or directory.
///
/// Uses `blake3` hashing algorithm.
///
/// ```
/// use paq;
///
/// let source = std::path::PathBuf::from("example");
/// let ignore_hidden = true;
/// let source_hash: paq::ArrayString<64> = paq::try_hash_source(&source, ignore_hidden).unwrap();
///
/// assert_eq!(&source_hash[..], "a593d18de8b696c153df9079c662346fafbb555cc4b2bbf5c7e6747e23a24d74");
/// ```
pub fn try_hash_source(source: &Path, ignore_hidden: bool) -> Result<ArrayString<64>, Error> {
    // construct file system walker
    let mut walker = WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_entry(filter(ignore_hidden));

    // construct iterator that retrieves system path batches using walker
    let batch_iter = iter::from_fn(move || {
        let mut batch = Vec::with_capacity(PATH_BATCH_SIZE);
        for _ in 0..PATH_BATCH_SIZE {
            match walker.next() {
                Some(Ok(entry)) => batch.push(Ok(entry)),
                Some(Err(error)) => {
                    batch.push(Err(Error::Walk(error)));
                    break;
                }
                None => break,
            }
        }
        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    });

    // run hashing pipeline using parallel batching
    let mut hashes: Vec<[u8; 32]> = batch_iter
        .par_bridge()
        .flat_map_iter(|batch| {
            batch
                .into_iter()
                .map(|entry| try_hash_path(source, &entry?))
        })
        .collect::<Result<_, Error>>()?;

    // parallel sort using default rayon MAX_SEQUENTIAL threshold (2k items)
    hashes.par_sort_unstable();

    Ok(get_hashes_root(hashes))
}

/// Hash file system source, panicking on error.
pub fn hash_source(source: &Path, ignore_hidden: bool) -> ArrayString<64> {
    try_hash_source(source, ignore_hidden).unwrap()
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    fn test_directory(name: &str) -> super::PathBuf {
        let path = std::env::temp_dir().join("paq").join(name);
        if path.exists() {
            super::fs::remove_dir_all(&path).unwrap();
        }
        super::fs::create_dir_all(&path).unwrap();
        path
    }

    fn file_entry(path: &super::Path) -> super::DirEntry {
        super::WalkDir::new(path.parent().unwrap())
            .into_iter()
            .find_map(|entry| {
                let entry = entry.unwrap();
                (entry.path() == path).then_some(entry)
            })
            .unwrap()
    }

    #[test]
    fn it_hashes_files_by_size() {
        let file_sizes = vec![
            ("empty", 0),
            ("buffered", super::MAX_FILE_SIZE_FOR_UNBUFFERED_READ),
        ];
        #[cfg(not(target_os = "windows"))]
        let file_sizes = {
            let mut file_sizes = file_sizes;
            file_sizes.push(("memory-mapped", super::MIN_FILE_SIZE_FOR_MMAP_READ + 1));
            file_sizes
        };
        let dir = test_directory("it_hashes_files_by_size");

        for (file_name, file_size) in file_sizes {
            let file_contents = vec![0; file_size as usize];
            let path = dir.join(file_name);
            super::fs::write(&path, &file_contents).unwrap();
            let entry = file_entry(&path);

            let hash = super::try_hash_path(&dir, &entry).unwrap();
            let mut hasher = super::Hasher::new();
            hasher.update(file_name.as_bytes());
            hasher.update(&file_contents);
            assert_eq!(hash, *hasher.finalize().as_bytes());
        }

        super::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn it_returns_io_error_for_missing_path() {
        let path = super::Path::new(env!("CARGO_MANIFEST_DIR")).join("__paq_test_missing_path__");
        let mut hasher = super::Hasher::new();

        let error = super::try_buffer_file_to_hasher(&mut hasher, &path).unwrap_err();
        assert!(error
            .to_string()
            .starts_with(format!("failed to access path `{}`:", path.display()).as_str()));
        assert!(matches!(
            error,
            super::Error::Io {
                path: error_path,
                ..
            } if error_path == path
        ));
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn it_returns_io_error_for_directory_read() {
        let dir = test_directory("it_returns_io_error_for_directory_read");
        let mut hasher = super::Hasher::new();

        let error = super::try_buffer_file_to_hasher(&mut hasher, &dir).unwrap_err();
        assert!(matches!(
            error,
            super::Error::Io {
                path: error_path,
                ..
            } if error_path == dir
        ));

        super::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn it_returns_error_for_path_outside_source() {
        let source = super::Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = source.join("__paq_test_source__");
        let entry = super::WalkDir::new(source)
            .into_iter()
            .next()
            .unwrap()
            .unwrap();

        let error = super::try_hash_path(&root, &entry).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "path `{}` is outside source `{}`",
                source.display(),
                root.display()
            )
        );
        assert!(matches!(
            error,
            super::Error::OutsideSource {
                path: error_path,
                root: error_root,
            } if error_path == source && error_root == root
        ));
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn it_returns_io_error_for_missing_symlink() {
        use std::os::unix::fs::symlink;

        let dir = test_directory("it_returns_io_error_for_missing_symlink");
        let path = dir.join("link");
        symlink("target", &path).unwrap();
        let entry = file_entry(&path);
        super::fs::remove_file(&path).unwrap();

        let error = super::try_hash_path(&dir, &entry).unwrap_err();
        assert!(matches!(
            error,
            super::Error::Io {
                path: error_path,
                ..
            } if error_path == path
        ));

        super::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn it_returns_error_for_invalid_utf8_symlink() {
        use std::{
            ffi::OsString,
            os::{unix::ffi::OsStringExt, unix::fs::symlink},
        };

        let dir = test_directory("it_returns_error_for_invalid_utf8_symlink");
        let target = OsString::from_vec(vec![0xff]);
        let path = dir.join("link");
        symlink(&target, &path).unwrap();
        let entry = file_entry(&path);

        let error = super::try_hash_path(&dir, &entry).unwrap_err();
        assert!(matches!(
            error,
            super::Error::InvalidUtf8Path(error_path) if error_path == target
        ));

        super::fs::remove_dir_all(dir).unwrap();
    }
}
