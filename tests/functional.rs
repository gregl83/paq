use std::env;
use std::path::PathBuf;
use paq::hash_source;

mod utils;

use utils::TempDir;

#[test]
fn it_hashes_directory() {
    let expectation = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let dir = TempDir::new("it_hashes_directory").unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(hash_not_ignored, expectation);
}

#[test]
fn it_hashes_directory_from_any_path() {
    let expectation = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let dir = TempDir::new("it_hashes_directory_from_any_path").unwrap();
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
fn it_hashes_directory_symlink_without_following() {
    let expectation = "778e05e084f80adb22043d7e45ad7236e1fe07bfb9983f5f26867c205dcc0112";

    let symlink_name = "symlink";
    let symlink_target = env::current_dir().unwrap();
    let dir = TempDir::new("it_hashes_directory_symlink_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(hash_ignored, expectation);
}

#[test]
fn it_hashes_single_file() {
    let expectation = "357dc7acd9dd411e9bd92945cad58062559910dbc229a7f86a21687e626a955e";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_hashes_single_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let file_directory = dir.path().join(file_name);
    let source = file_directory.as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(hash_not_ignored, expectation);
}