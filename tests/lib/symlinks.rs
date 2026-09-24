use crate::utils::TempDir;
use std::path::PathBuf;

#[cfg(target_family = "unix")]
#[test]
fn it_hashes_directory_relative_symlink_without_following() {
    let expectation = "5bb837eff87dee38d63c081bc30a8d0ce7cc871c8b32e38e3e40f9ccdef4db98";

    let symlink_name = "symlink";
    let symlink_target = PathBuf::from("target");
    let dir = TempDir::new("it_hashes_directory_relative_symlink_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_ignored[..], expectation);
}

#[cfg(target_family = "unix")]
#[test]
fn it_hashes_directory_absolute_symlink_without_following() {
    let expectation = "60fbb028703074fe8b17a20155696f6dffab6a3de071b716792350f946d917aa";

    let symlink_name = "symlink";
    let symlink_target = PathBuf::from("/");
    let dir = TempDir::new("it_hashes_directory_absolute_symlink_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_ignored[..], expectation);
}
