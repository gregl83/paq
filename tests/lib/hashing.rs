use crate::utils::TempDir;
use std::{env, path::PathBuf};

#[test]
fn it_hashes_single_file() {
    let expectation = "31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_hashes_single_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_not_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_not_ignored[..], expectation);
}

#[test]
fn it_hashes_directory() {
    let expectation = "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

    let dir = TempDir::new("it_hashes_directory").unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_not_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_not_ignored[..], expectation);
}

#[test]
fn it_hashes_directory_from_any_path() {
    let expectation = "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

    let dir = TempDir::new("it_hashes_directory_from_any_path").unwrap();
    let source = dir.path().canonicalize().unwrap();
    let original_path = env::current_dir().unwrap();
    let new_path = PathBuf::from("/");

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_ignored[..], expectation);

    env::set_current_dir(new_path).unwrap();

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_ignored[..], expectation);

    env::set_current_dir(original_path).unwrap();
}

#[test]
fn it_hashes_directory_with_file() {
    let expectation = "3f415b1c522892b37d3f945e8427a98bd2cbc50a2f857cb0c8c70ef979d4a112";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_hashes_directory_with_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_not_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_not_ignored[..], expectation);
}

#[test]
fn it_hashes_directory_files_consistently() {
    let expectation = "12d64568d500f78a043019d8814311c07916abba0653b778a2a7c536b0e3f316";

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
    dir.new_file(charlie_file_name, charlie_file_contents)
        .unwrap();
    dir.new_file(one_file_name, one_file_contents).unwrap();
    dir.new_file(nine_file_name, nine_file_contents).unwrap();
    let source = dir.path().canonicalize().unwrap();

    for _ in 0..50 {
        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_not_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_not_ignored[..], expectation);
    }
}
