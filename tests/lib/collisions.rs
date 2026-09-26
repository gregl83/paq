#[cfg(target_family = "unix")]
use std::path::PathBuf;

use crate::utils::{
    assert_distinct_tree_hashes,
    TempDir,
};

#[cfg(target_family = "unix")]
#[test]
fn it_distinguishes_symlink_target_from_file_contents() {
    let link = TempDir::new("collision_symlink_file_link").unwrap();
    let file = TempDir::new("collision_symlink_file_file").unwrap();
    // The target deliberately does not exist: hash its text, not contents.
    link.new_symlink("a", PathBuf::from("bc")).unwrap();
    file.new_file("a", b"bc").unwrap();

    assert_distinct_tree_hashes(&link, &file);
}

#[cfg(target_family = "unix")]
#[test]
fn it_distinguishes_symlink_path_target_boundaries() {
    let left = TempDir::new("collision_symlink_boundary_left").unwrap();
    let right = TempDir::new("collision_symlink_boundary_right").unwrap();
    // Without field boundaries, both path/target pairs concatenate to "abc".
    left.new_symlink("a", PathBuf::from("bc")).unwrap();
    right.new_symlink("ab", PathBuf::from("c")).unwrap();

    assert_distinct_tree_hashes(&left, &right);
}

#[cfg(target_family = "unix")]
#[test]
fn it_distinguishes_symlink_from_empty_directory() {
    let link = TempDir::new("collision_symlink_directory_link").unwrap();
    let directory = TempDir::new("collision_symlink_directory_directory").unwrap();
    // Without entry types or field boundaries, both entries encode as "ab".
    link.new_symlink("a", PathBuf::from("b")).unwrap();
    std::fs::create_dir(directory.path().join("ab")).unwrap();

    assert_distinct_tree_hashes(&link, &directory);
}

#[test]
fn it_distinguishes_file_path_content_boundaries() {
    let left = TempDir::new("collision_file_boundary_left").unwrap();
    let right = TempDir::new("collision_file_boundary_right").unwrap();
    left.new_file("a", b"bc").unwrap();
    right.new_file("ab", b"c").unwrap();

    assert_distinct_tree_hashes(&left, &right);
}

#[test]
fn it_distinguishes_empty_file_from_empty_directory() {
    let file = TempDir::new("collision_empty_file").unwrap();
    let directory = TempDir::new("collision_empty_directory").unwrap();
    file.new_file("a", b"").unwrap();
    std::fs::create_dir(directory.path().join("a")).unwrap();

    assert_distinct_tree_hashes(&file, &directory);
}
