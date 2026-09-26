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
// Allow work stealing within each batch while amortizing scheduling.
const HASH_BATCH_MIN_LEN: usize = 4;
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
    // encode relative path, a combined NUL/type marker, then optional payload (collision resistance)
    let entry_type = if source_type.is_file() {
        0x01
    } else if source_type.is_dir() {
        0x02
    } else if source_type.is_symlink() {
        0x03
    } else {
        0x04
    };
    // hash paths for fs changes other than file content (must be relative to root)
    #[cfg(target_family = "unix")]
    {
        hasher.update(source_path.as_bytes());
    }
    #[cfg(target_family = "windows")]
    {
        hasher.update(source_path.replace("\\", "/").as_bytes());
    }
    hasher.update(&[0, entry_type]);
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

fn next_path_batch<T>(
    entries: &mut impl Iterator<Item = Result<T, Error>>,
) -> Option<Vec<Result<T, Error>>> {
    let mut batch = Vec::with_capacity(PATH_BATCH_SIZE);
    for _ in 0..PATH_BATCH_SIZE {
        match entries.next() {
            Some(Ok(entry)) => batch.push(Ok(entry)),
            Some(Err(error)) => {
                batch.push(Err(error));
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
}

/// Hash system source directory or file with `BLAKE3`.
///
/// Source **must** be a path to a directory or file.
///
/// Each entry hashes its relative path, a NUL byte, its type byte
/// (file: 1, directory: 2, symlink: 3, other: 4), and its payload. File payloads are
/// contents; symlink payloads are target paths. Other entries have no payload.
/// Sorted entry digests are concatenated and hashed to produce the source hash.
///
/// ```
/// use paq;
///
/// let source = std::path::PathBuf::from("example");
/// let ignore_hidden = true;
/// let source_hash: paq::ArrayString<64> = paq::try_hash_source(&source, ignore_hidden).unwrap();
///
/// assert_eq!(&source_hash[..], "2d7ba6963c4836dcbd679607bc432afce5e3c4ef1dc0bed145c20d1b8e2bda77");
/// ```
pub fn try_hash_source(source: &Path, ignore_hidden: bool) -> Result<ArrayString<64>, Error> {
    // construct file system walker
    let mut walker = WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_entry(filter(ignore_hidden))
        .map(|entry| entry.map_err(Error::Walk));

    // construct iterator that retrieves system path batches using walker
    let batch_iter = iter::from_fn(move || next_path_batch(&mut walker));

    // run hashing pipeline using parallel batching
    let mut hashes: Vec<[u8; 32]> = batch_iter
        .par_bridge()
        .flat_map(|batch| {
            batch
                .into_par_iter()
                .with_min_len(HASH_BATCH_MIN_LEN)
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
    fn it_bounds_batches_and_retains_walk_errors() {
        let dir = test_directory("batch_walk_errors");
        for error_index in [
            0,
            super::PATH_BATCH_SIZE - 1,
            super::PATH_BATCH_SIZE,
            super::PATH_BATCH_SIZE + 1,
        ] {
            let walk_error = super::WalkDir::new(dir.join("missing"))
                .into_iter()
                .next()
                .unwrap()
                .unwrap_err();
            let mut entries: Vec<Result<usize, super::Error>> = (0..error_index).map(Ok).collect();
            entries.push(Err(super::Error::Walk(walk_error)));
            entries.push(Ok(error_index + 1));
            let mut entries = entries.into_iter();
            let mut consumed = 0;
            let mut errors = 0;
            while let Some(batch) = super::next_path_batch(&mut entries) {
                assert!(batch.len() <= super::PATH_BATCH_SIZE);
                for (index, entry) in batch.iter().enumerate() {
                    if let Err(error) = entry {
                        assert!(matches!(error, super::Error::Walk(_)));
                        assert_eq!(index + 1, batch.len(), "an error ends its batch");
                        errors += 1;
                    }
                }
                consumed += batch.len();
            }
            assert_eq!(consumed, error_index + 2);
            assert_eq!(errors, 1);
        }
        super::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn it_hashes_batch_boundaries_with_one_and_multiple_workers() {
        let pools: Vec<_> = [1, 4]
            .into_iter()
            .map(|workers| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(workers)
                    .build()
                    .unwrap()
            })
            .collect();
        for count in [98, 99, 100, 101, 199, 200] {
            let dir = test_directory(&format!("batch_boundary_{count}"));
            let mut expected = vec![*blake3::hash(&[0, 0x02]).as_bytes()];
            for index in 0..count {
                let name = format!("file-{index:03}");
                // Cluster medium payloads at the beginning of each batch.
                let size = if index % super::PATH_BATCH_SIZE < 12 {
                    64 * 1024
                } else {
                    index % 2
                };
                let contents = vec![(index % 251) as u8; size];
                super::fs::write(dir.join(&name), &contents).unwrap();
                let mut hasher = super::Hasher::new();
                hasher.update(name.as_bytes());
                hasher.update(&[0, 0x01]);
                hasher.update(&contents);
                expected.push(*hasher.finalize().as_bytes());
            }
            expected.sort_unstable();
            let expected = super::get_hashes_root(expected);
            for pool in &pools {
                for ignore_hidden in [false, true] {
                    assert_eq!(
                        pool.install(|| super::try_hash_source(&dir, ignore_hidden))
                            .unwrap(),
                        expected
                    );
                }
            }
            super::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn it_propagates_hashing_errors_across_parallel_batches() {
        use std::{
            ffi::OsString,
            os::unix::{ffi::OsStringExt, fs::symlink},
        };

        let dir = test_directory("batch_hashing_error");
        for index in 0..2 * super::PATH_BATCH_SIZE {
            super::fs::write(dir.join(format!("file-{index:03}")), b"payload").unwrap();
        }
        let target = OsString::from_vec(vec![0xff]);
        for name in ["aaa-link", "mmm-link", "zzz-link"] {
            let path = dir.join(name);
            symlink(&target, &path).unwrap();
            for workers in [1, 4] {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(workers)
                    .build()
                    .unwrap();
                let error = pool
                    .install(|| super::try_hash_source(&dir, false))
                    .unwrap_err();
                assert!(
                    matches!(error, super::Error::InvalidUtf8Path(error_path) if error_path == super::Path::new(&target))
                );
            }
            super::fs::remove_file(path).unwrap();
        }
        super::fs::remove_dir_all(dir).unwrap();
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
            hasher.update(&[0, 0x01]);
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
