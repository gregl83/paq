use std::{
    fs,
    io::prelude::*,
    path::{
        Path,
        PathBuf,
    },
};

pub use arrayvec::ArrayString;
use blake3::Hasher;
use memmap2::Mmap;
use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};


pub const MAX_FILE_SIZE_FOR_UNBUFFERED_READ: u64 = 1024 + 1;
#[cfg(not(target_os = "windows"))]
pub const MIN_FILE_SIZE_FOR_MMAP_READ: u64 = 1024 * 1024 - 1;
#[cfg(target_os = "windows")]
pub const MIN_FILE_SIZE_FOR_MMAP_READ: u64 = 1024 * 1024 * 1024 - 1;
#[cfg(not(target_os = "windows"))]
pub const FILE_BUFFER_SIZE: usize = 32 * 1024;
#[cfg(target_os = "windows")]
pub const FILE_BUFFER_SIZE: usize = 128 * 1024;

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

fn get_paths(root: &Path, ignore_hidden: bool) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(filter(ignore_hidden))
        .par_bridge() // Convert the iterator to a parallel iterator
        .fold(
            Vec::new,
            |mut acc, entry| {
                if let Ok(entry) = entry {
                    acc.push(entry.into_path());
                }
                acc
            },
        )
        .reduce(
            Vec::new,
            |mut paths_a, paths_b| {
                paths_a.extend(paths_b);
                paths_a
            },
        )
}

fn buffer_file_to_hasher(hasher: &mut Hasher, path: &str) {
    let mut file = fs::File::open(path).unwrap();
    let mut buffer = [0; FILE_BUFFER_SIZE];
    loop {
        let buffer_size = file.read(&mut buffer[..]).unwrap();
        if buffer_size == 0 { break; }
        hasher.update(&buffer[..buffer_size]);
    }
}

fn hash_path(root: &Path, path: PathBuf) -> [u8; 32] {
    let mut hasher = Hasher::new();
    let source_path = path.strip_prefix(root).unwrap().to_str().unwrap();
    // hash  paths for fs changes other than file content (must be relative to root)
    #[cfg(target_family = "unix")]
    {
        hasher.update(source_path.as_bytes());
    }
    #[cfg(target_family = "windows")]
    {
        hasher.update(source_path.replace("\\", "/").as_bytes());
    }
    let relative_path = path.as_os_str().to_str().unwrap();
    let metadata = fs::symlink_metadata(relative_path).unwrap();
    if metadata.is_symlink() {
        // for symlinks add hash of target path
        let symlink_target = fs::read_link(relative_path).unwrap();
        #[cfg(target_family = "unix")]
        {
            hasher.update(symlink_target.to_str().unwrap().as_bytes());
        }
        #[cfg(target_family = "windows")]
        {
            hasher.update(
                symlink_target
                    .to_str()
                    .unwrap()
                    .replace("\\", "/")
                    .as_bytes(),
            );
        }
    } else if metadata.is_file() {
        // for files, add contents to hasher
        let file_size = metadata.len();
        if file_size == 0 {
            // empty file, return immediately
            return *hasher.finalize().as_bytes();
        } else if file_size < MAX_FILE_SIZE_FOR_UNBUFFERED_READ {
            // small file read using unbuffered
            let file = fs::read(&path).unwrap();
            hasher.update(&file);
        } else if file_size > MIN_FILE_SIZE_FOR_MMAP_READ {
            // large size files read using mmap or fail to buffered read
            let file = fs::File::open(&path).unwrap();
            match unsafe { Mmap::map(&file) } {
                Ok(mmap) => { hasher.update(&mmap); },
                Err(_) => { buffer_file_to_hasher(&mut hasher, relative_path); },
            }
        } else {
            // medium file size read using buffer
            buffer_file_to_hasher(&mut hasher, relative_path);
        }
    }
    *hasher.finalize().as_bytes()
}

fn hash_paths(root: &Path, paths: Vec<PathBuf>) -> Vec<[u8; 32]> {
    let mut hashes: Vec<_> = paths
        .into_par_iter()
        .map(|path| hash_path(root, path))
        .collect();
    // parallel sort using default rayon MAX_SEQUENTIAL threshold (2k items)
    hashes.par_sort_unstable();
    hashes
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
/// let source_hash: paq::ArrayString<64> = paq::hash_source(&source, ignore_hidden);
///
/// assert_eq!(&source_hash[..], "d7d25c9b2fdb7391e650085a985ad0d892c7f0dd5edd32c7ccdb4b0d1c34c430");
/// ```
pub fn hash_source(source: &Path, ignore_hidden: bool) -> ArrayString<64> {
    let paths = get_paths(source, ignore_hidden);
    let hashes = hash_paths(source, paths);
    get_hashes_root(hashes)
}
