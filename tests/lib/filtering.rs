use crate::utils::TempDir;

#[test]
fn it_hashes_directory_with_ignored_file() {
    let expectation_not_ignored =
        "1e8fcf474e6cad4ed136fe2286b5670765510a2c96505ed7a58ad96c01f4f433";
    let expectation_ignored =
        "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

    let file_name = ".ignored";
    let file_contents = ".ignored-body".as_bytes();
    let dir = TempDir::new("it_hashes_directory_with_ignored_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation_ignored);
    let hash_not_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_not_ignored[..], expectation_not_ignored);
}

#[test]
fn it_hashes_directory_with_ignored_subdirectory() {
    let expectation_not_ignored =
        "2ba515752a46e2d4e5ce37bd8dc4f9e0704461fbf4d90aa02bd2e155a1504167";
    let expectation_ignored =
        "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

    let dir = TempDir::new("it_hashes_directory_with_ignored_subdirectory").unwrap();
    let source = dir.path().canonicalize().unwrap();

    let subdir = TempDir::new("it_hashes_directory_with_ignored_subdirectory/.test").unwrap();
    let hash_ignored = paq::hash_source(&source, true);
    assert_eq!(&hash_ignored[..], expectation_ignored);
    let hash_not_ignored = paq::hash_source(&source, false);
    assert_eq!(&hash_not_ignored[..], expectation_not_ignored);

    println!(
        "prevent early subdir drop for: {}",
        subdir.path().as_os_str().to_str().unwrap()
    )
}
