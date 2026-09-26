use std::{
    env,
    error::Error,
};

use merkle_hash::{
    Algorithm,
    Encodable,
    MerkleTree,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let source = args.nth(1).ok_or("usage: merkle-hash <directory>")?;
    if args.next().is_some() {
        return Err("usage: merkle-hash <directory>".into());
    }
    let tree = MerkleTree::builder(&source)
        .algorithm(Algorithm::Blake3)
        .hash_names(true)
        .build()?;
    println!("{}", tree.root.item.hash.to_hex_string());
    Ok(())
}
