use crate::utils::TempDir;

#[test]
fn it_hashes_directory_with_ignored_file() {
    let expectation_not_ignored =
        "e383192a5ef45576817b4222e455e3d538ae3bab279a62c0a8b67279ad007072";
    let expectation_ignored =
        "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

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
        "f38a56a87aca98131b2fa5914fd13bc11f5823602293e8d84b5c69000b33ebf2";
    let expectation_ignored =
        "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

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
