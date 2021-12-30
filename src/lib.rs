use std::fs;
use std::collections::BTreeMap;
use walkdir::{WalkDir, DirEntry};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

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

fn get_paths(root: &str, ignore_hidden: bool) -> BTreeMap<String, (String, bool)> {
    // BTreeMap ensures order of files/directories by path
    let mut paths = BTreeMap::<String, (String, bool)>::new();

    for entry in WalkDir::new(root)
        .follow_links(true)
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
        paths.insert(relative_path, (absolute_path, path.is_file()));
    }
    paths
}

fn hash_paths(paths: BTreeMap<String, (String, bool)>) -> Vec<[u8; 32]> {
    let mut hashes = Vec::<[u8; 32]>::new();

    for (relative_path, (absolute_path, is_file)) in paths {
        // hash paths for fs changes other than file content (must be relative to root)
        hashes.push(Sha256::hash(relative_path.as_bytes()));
        // for files add hash of contents
        if is_file {
            let file_bytes = fs::read(absolute_path).unwrap();
            hashes.push(Sha256::hash(file_bytes.as_slice()))
        }
    }

    hashes
}

fn get_hashes_root(file_hashes: Vec<[u8; 32]>) -> Option<String> {
    MerkleTree::<Sha256>::from_leaves(&file_hashes).root_hex()
}

/// Hash file system source.
///
/// Source **must** be a path to a file or directory.
///
/// Uses `SHA256` hashing algorithm via `Merkle Tree`.
///
/// ```ignore
/// use paq;
///
/// let source = "/path/to/source";
/// let ignore_hidden = true;
/// let source_hash: String = paq::hash_source(source, ignore_hidden);
///
/// println!("{}", source_hash);
/// ```
pub fn hash_source(source: &str, ignore_hidden: bool) -> String {
    let paths = get_paths(source, ignore_hidden);
    let hashes = hash_paths(paths);
    get_hashes_root(hashes).unwrap()
}