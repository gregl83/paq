use crate::utils::TempDir;
use assert_cmd::{cargo::cargo_bin, Command};

#[test]
fn it_ignores_hidden_files_using_short_arg() {
    let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let file_name = ".ignored";
    let file_contents = ".ignored-body".as_bytes();
    let dir = TempDir::new("it_ignores_hidden_files_using_short_arg").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg("-i")
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();
}

#[test]
fn it_ignores_hidden_files_using_long_arg() {
    let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let file_name = ".ignored";
    let file_contents = ".ignored-body".as_bytes();
    let dir = TempDir::new("it_ignores_hidden_files_using_long_arg").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg("--ignore-hidden")
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();
}
