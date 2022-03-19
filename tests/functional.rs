use std::env;
use std::path::PathBuf;
use paq::hash_source;

mod utils;

use utils::TempDir;

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
fn it_hashes_directory_with_file() {
    let expectation = "1488f4fcc0eed29cd5fdba38b202f798b8cd7d4541849cd0a7969334b399477e";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_hashes_directory_with_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(hash_not_ignored, expectation);
}

#[test]
fn it_hashes_directory_with_ignored_file() {
    let expectation_not_ignored = "8830a5da097b494a288e91963a887a524ca0fcf94d1fd7fc139a78d890987bca";
    let expectation_ignored = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    let file_name = ".ignored";
    let file_contents = ".ignored-body".as_bytes();
    let dir = TempDir::new("it_hashes_directory_with_ignored_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(hash_ignored, expectation_ignored);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(hash_not_ignored, expectation_not_ignored);
}

#[test]
fn it_hashes_directory_files_consistently() {
    let expectation = "2f79669ee737bc3cd206e56066f99c81f691c8103e7a7bed04f735fe65d06b99";

    let alpha_file_name = "alpha";
    let alpha_file_contents = "alpha-body".as_bytes();
    let bravo_file_name = "bravo";
    let bravo_file_contents = "bravo-body".as_bytes();
    let charlie_file_name = "charlie";
    let charlie_file_contents = "charlie-body".as_bytes();
    let one_file_name = "1";
    let one_file_contents = "1-body".as_bytes();
    let nine_file_name = "9";
    let nine_file_contents = "9-body".as_bytes();

    let dir = TempDir::new("it_hashes_directory_files_consistently").unwrap();
    dir.new_file(alpha_file_name, alpha_file_contents).unwrap();
    dir.new_file(bravo_file_name, bravo_file_contents).unwrap();
    dir.new_file(charlie_file_name, charlie_file_contents).unwrap();
    dir.new_file(one_file_name, one_file_contents).unwrap();
    dir.new_file(nine_file_name, nine_file_contents).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    for _ in 0..50 {
        let hash_ignored = paq::hash_source(source, true);
        assert_eq!(hash_ignored, expectation);
        let hash_not_ignored = paq::hash_source(source, false);
        assert_eq!(hash_not_ignored, expectation);
    }
}