use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use rs_merkle::{algorithms::Sha256, Error, Hasher, MerkleTree};

fn get_file_paths(root: &str) -> Vec<String> {
    let mut file_paths = Vec::<String>::new();
    for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            let canonical_file_path = fs::canonicalize(entry.path()).unwrap();
            file_paths.push(String::from(canonical_file_path.as_os_str().to_str().unwrap()));
        }
    }
    file_paths
}

fn hash_file_paths(file_paths: Vec<String>) -> Vec<[u8; 32]> {
    let mut hashes = Vec::<[u8; 32]>::new();

    for file_path in file_paths {
        let file_bytes = fs::read(file_path).unwrap();
        hashes.push(Sha256::hash(file_bytes.as_slice()))
    }

    hashes
}

fn get_file_hashes_root(file_hashes: Vec<[u8; 32]>) -> Option<String> {
    MerkleTree::<Sha256>::from_leaves(&file_hashes).root_hex()
}

fn main() {
    // todo - add clap with dir argument
    // todo - handle single file and multiple files (dir path)
    // todo - add error handling with messaging
    let file_paths = get_file_paths(".");
    let file_hashes = hash_file_paths(file_paths);
    let root = get_file_hashes_root(file_hashes).unwrap();
    println!("{}", root.as_str());
}
