use std::path::PathBuf;

mod utils;

use utils::TempDir;

#[test]
fn it_adds_two() {
    println!("HERE");
    let dir = TempDir::new().unwrap();
    println!("{:?}", dir.path().as_os_str());
    assert_eq!(4, 3);
}