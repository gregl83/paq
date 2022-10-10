use std::env;
use std::path::PathBuf;

mod utils;
use utils::TempDir;

#[test]
fn it_hashes_single_file() {
    let expectation = "219a256c060e2fc673e4dd62f8ade3acb5649052c2032ac313b4d7b60eda6eb4";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_hashes_single_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let file_directory = dir.path().join(file_name);
    let source = file_directory.as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_not_ignored[..], expectation);
}

#[test]
fn it_hashes_directory() {
    let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let dir = TempDir::new("it_hashes_directory").unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_not_ignored[..], expectation);
}

#[test]
fn it_hashes_directory_from_any_path() {
    let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let dir = TempDir::new("it_hashes_directory_from_any_path").unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();
    let original_path = env::current_dir().unwrap();
    let new_path = PathBuf::from("/");

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_ignored[..], expectation);

    env::set_current_dir(new_path).unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_ignored[..], expectation);

    env::set_current_dir(original_path).unwrap();
}

#[test]
fn it_hashes_directory_symlink_without_following() {
    let expectation = "47e609d5f708cfef0ddcc7f8f0f6226b63c93e9a0478bdda672e334cc020c70e";

    let symlink_name = "symlink";
    let symlink_target = env::current_dir().unwrap();
    let dir = TempDir::new("it_hashes_directory_symlink_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_ignored[..], expectation);
}

#[test]
fn it_hashes_directory_with_file() {
    let expectation = "5a9755e11702b86557541d1d5d9d24212e6b1d4d2a1d09312dd96f0dfa92654b";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_hashes_directory_with_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_not_ignored[..], expectation);
}

#[test]
fn it_hashes_directory_with_ignored_file() {
    let expectation_not_ignored = "5eef2d7a4bf361cf3f89576d65db7ef2fb4e2745b6c460a96c6ceec07aa14a42";
    let expectation_ignored = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let file_name = ".ignored";
    let file_contents = ".ignored-body".as_bytes();
    let dir = TempDir::new("it_hashes_directory_with_ignored_file").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation_ignored);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_not_ignored[..], expectation_not_ignored);
}

#[test]
fn it_hashes_directory_with_ignored_subdirectory() {
    let expectation_not_ignored = "f38a56a87aca98131b2fa5914fd13bc11f5823602293e8d84b5c69000b33ebf2";
    let expectation_ignored = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let dir = TempDir::new("it_hashes_directory_with_ignored_subdirectory").unwrap();
    let source = dir.path().as_os_str().to_str().unwrap();

    let subdir = TempDir::new("it_hashes_directory_with_ignored_subdirectory/.test").unwrap();
    let hash_ignored = paq::hash_source(source, true);
    assert_eq!(&hash_ignored[..], expectation_ignored);
    let hash_not_ignored = paq::hash_source(source, false);
    assert_eq!(&hash_not_ignored[..], expectation_not_ignored);

    println!("prevent early subdir drop for: {}", subdir.path().as_os_str().to_str().unwrap())
}

#[test]
fn it_hashes_directory_files_consistently() {
    let expectation = "50ec0be65febd8b1c22723b7c0e17901c6b0e9a3719364b8d592e82159ba0280";

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
        assert_eq!(&hash_ignored[..], expectation);
        let hash_not_ignored = paq::hash_source(source, false);
        assert_eq!(&hash_not_ignored[..], expectation);
    }
}