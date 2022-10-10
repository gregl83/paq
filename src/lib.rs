use std::fs;
use std::io::prelude::*;
use arrayvec::ArrayString;
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

fn get_paths(root: &str, ignore_hidden: bool) -> Vec<(String, String, bool)> {
    let mut paths= Vec::<(String, String, bool)>::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(filter(ignore_hidden))
    {
        let directory_entry = entry.unwrap();
        let path = directory_entry.path();
        let relative_path = String::from(
            path.strip_prefix(&root).unwrap().to_str().unwrap()
        );
        let absolute_path = String::from(
            path.as_os_str().to_str().unwrap()
        );
        paths.push((relative_path, absolute_path, path.is_file()))
    }
    paths
}

fn hash_paths(paths: Vec<(String, String, bool)>) -> Vec<[u8; 32]> {
    let mut hashes: Vec<_> = paths.into_par_iter().map(|(relative_path, absolute_path, is_file)| {
        let mut hasher = blake3::Hasher::new();
        // hash paths for fs changes other than file content (must be relative to root)
        hasher.update(relative_path.as_bytes());
        // for files add hash of contents
        if is_file {
            let mut f = fs::File::open(absolute_path).unwrap();
            let mut buffer = [0; 4096];
            loop {
                let n = f.read(&mut buffer[..]).unwrap();
                if n == 0 {
                    break
                }
                hasher.update(buffer.as_slice());
            }
        }
        *hasher.finalize().as_bytes()
    }).collect();
    hashes.sort();
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
/// use arrayvec::ArrayString;
///
/// let source = "example";
/// let ignore_hidden = true;
/// let source_hash: ArrayString<64> = paq::hash_source(source, ignore_hidden);
///
/// assert_eq!(&source_hash[..], "778c013fbdb4d129357ec8023ea1d147e60a014858cfc2dd998af6c946e802a9");
/// ```
pub fn hash_source(source: &str, ignore_hidden: bool) -> ArrayString<64> {
    let paths = get_paths(source, ignore_hidden);
    let hashes = hash_paths(paths);
    get_hashes_root(hashes)
}