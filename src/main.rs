use std::fs;
use std::collections::BTreeMap;
use walkdir::WalkDir;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use clap::{App, Arg};

fn get_paths(root: &str) -> BTreeMap<String, (String, bool)> {
    // BTreeMap ensures order of files/directories by path
    let mut paths = BTreeMap::<String, (String, bool)>::new();

    for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        let path = entry.path();
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

fn main() {
    // todo - add error handling with messaging

    let matches = App::new("paq")
        .version("0.3.2")
        .about("paq files to hash.")
        .arg(Arg::with_name("src")
            .help("Source to hash (path)")
            .default_value(".")
            .index(1))
        .get_matches();

    let root = matches.value_of("src").unwrap();
    let paths = get_paths(root);
    let hashes = hash_paths(paths);
    let root = get_hashes_root(hashes).unwrap();
    println!("{}", root.as_str());
}
