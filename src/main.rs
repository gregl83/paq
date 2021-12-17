use std::fs;
use std::path::Path;
use std::collections::BTreeMap;
use walkdir::WalkDir;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

fn get_paths(root: &str) -> BTreeMap<String, bool> {
    // BTreeMap ensures order of files/directories by path
    let mut paths = BTreeMap::<String, bool>::new();

    for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        let path = String::from(entry.path().as_os_str().to_str().unwrap());
        paths.insert(path, entry.path().is_file());
    }
    paths
}

fn hash_paths(paths: BTreeMap<String, bool>) -> Vec<[u8; 32]> {
    let mut hashes = Vec::<[u8; 32]>::new();

    for (absolute_path, is_file) in paths {
        // hash paths for fs changes other than file content
        hashes.push(Sha256::hash(absolute_path.as_bytes()));
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

fn main() {
    // todo - add clap with dir argument
    // todo - handle single file and multiple files (dir path)
    // todo - add error handling with messaging
    let file_paths = get_paths(".");
    let file_hashes = hash_paths(file_paths);
    let root = get_hashes_root(file_hashes).unwrap();
    println!("{}", root.as_str());
}
