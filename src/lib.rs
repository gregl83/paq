use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
pub use arrayvec::ArrayString;
use rayon::prelude::*;
use blake3::Hasher;
use walkdir::{WalkDir, DirEntry};

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s != "." && s.starts_with("."))
         .unwrap_or(false)
}

fn filter(ignore_hidden: bool) -> impl FnMut(&DirEntry) -> bool {
    if ignore_hidden {
        |entry: &DirEntry| -> bool { !is_hidden(entry) }
    } else {
        |_: &DirEntry| -> bool { true }
    }
}

fn get_paths(root: &PathBuf, ignore_hidden: bool) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(filter(ignore_hidden))
        .par_bridge() // Convert the iterator to a parallel iterator
        .fold(
            || Vec::new(),
            |mut acc, entry| {
                match entry {
                    Ok(entry) => {
                        acc.push(entry.into_path());
                    },
                    _ => {}
                }
                acc
            },
        )
        .reduce(
            || Vec::new(),
            |mut paths_a, paths_b| {
                paths_a.extend(paths_b.into_iter());
                paths_a
            },
        )
}

fn hash_paths(root: &PathBuf, paths: Vec<PathBuf>) -> Vec<[u8; 32]> {
    let mut hashes: Vec<_> = paths.into_par_iter().map(|path| {
        let mut hasher = Hasher::new();
        let relative_path = String::from(
            path.strip_prefix(&root).unwrap().to_str().unwrap()
        );
        // hash paths for fs changes other than file content (must be relative to root)
        hasher.update(relative_path.as_bytes());
        // for files add hash of contents
        if path.is_file() {
            let absolute_path = String::from(
                path.as_os_str().to_str().unwrap()
            );
            let mut f = fs::File::open(absolute_path).unwrap();
            let mut buffer = [0; 32768];
            loop {
                let n = f.read(&mut buffer[..]).unwrap();
                if n == 0 {
                    break
                }
                hasher.update(&buffer[..n]);
            }
        }
        *hasher.finalize().as_bytes()
    }).collect();
    hashes.sort_unstable();
    hashes
}

fn get_hashes_root(file_hashes: Vec<[u8; 32]>) -> ArrayString<64> {
    let mut hasher = Hasher::new();

    for file_hash in file_hashes {
        hasher.update(file_hash.as_slice());
    }

    hasher.finalize().to_hex()
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
/// assert_eq!(&source_hash[..], "494f366c528a930bb654b58721ab01683146381e1d2bf3e187311f9b725bfa19");
/// ```
pub fn hash_source(source: &PathBuf, ignore_hidden: bool) -> ArrayString<64> {
    let paths = get_paths(&source, ignore_hidden);
    let hashes = hash_paths(&source, paths);
    get_hashes_root(hashes)
}