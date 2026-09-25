use crate::utils::TempDir;
use std::path::PathBuf;

#[cfg(target_family = "unix")]
#[test]
fn it_hashes_directory_relative_symlink_without_following() {
    let expectation = "e2c989a91166ea125a275abc9832cef4ffd21953a5eeb31763f850a7aa7c7916";

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
    let expectation = "bef6e261b38ff07d099269e1783808888b85844c08c37cfff42722e1242bd7af";

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
