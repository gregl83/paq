use assert_cmd::{
    cargo::cargo_bin,
    Command,
};

use crate::utils::TempDir;

#[test]
fn it_ignores_hidden_files_using_short_arg() {
    let expectation = "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();
}

#[test]
fn it_ignores_hidden_files_using_long_arg() {
    let expectation = "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();
}
