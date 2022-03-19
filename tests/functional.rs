use std::env;
use std::path::PathBuf;
use paq::hash_source;

mod utils;

use utils::TempDir;

#[test]
fn it_hashes_empty_directory() {
    let expectation = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let dir = TempDir::new("it_hashes_empty_directory").unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(hash_not_ignored, expectation);
}

#[test]
fn it_hashes_from_any_path() {
    let expectation = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let dir = TempDir::new("it_hashes_from_any_path").unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();
    let original_path = env::current_dir().unwrap();
    let new_path = PathBuf::from("/");

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(hash_ignored, expectation);

    env::set_current_dir(new_path).unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(hash_ignored, expectation);

    env::set_current_dir(original_path).unwrap();
}

#[test]
fn it_hashes_symlinks_without_following() {
    let expectation = "778e05e084f80adb22043d7e45ad7236e1fe07bfb9983f5f26867c205dcc0112";

    let symlink_name = "symlink";
    let symlink_target = env::current_dir().unwrap();
    let dir = TempDir::new("it_hashes_symlinks_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(hash_ignored, expectation);
}