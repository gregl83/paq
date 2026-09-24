use crate::utils::TempDir;
use assert_cmd::{cargo::cargo_bin, Command};
use std::path::PathBuf;

#[test]
fn it_outputs_directory_hash_using_default_source() {
    let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let dir = TempDir::new("it_outputs_directory_hash_using_default_source").unwrap();

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .current_dir(dir.path())
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();
}

#[test]
fn it_outputs_file_hash_without_output() {
    let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let hash_file_name = "alpha.paq";
    let dir = TempDir::new("it_outputs_file_hash_without_output").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();

    assert!(!dir.path().join(hash_file_name).exists());
}

#[test]
fn it_outputs_file_hash_using_default_short_arg() {
    let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let hash_file_name = "alpha.paq";
    let dir = TempDir::new("it_outputs_file_hash_using_default_short_arg").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg("-o")
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_directory_hash_using_default_short_arg() {
    let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

    let source_name = "source";
    let hash_file_name = "source.paq";
    let dir = TempDir::new("it_outputs_directory_hash_using_default_short_arg").unwrap();
    let source = dir.path().join(source_name);
    std::fs::create_dir(&source).unwrap();

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg("-o")
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_file_hash_using_short_arg() {
    let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let hash_file_name = "custom.paq";
    let dir = TempDir::new("it_outputs_file_hash_using_short_arg").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);
    let mut output = PathBuf::from(&source).parent().unwrap().to_path_buf();
    output.push(hash_file_name);

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg(format!("-o={}", output.as_os_str().to_str().unwrap()))
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_file_hash_using_default_long_arg() {
    let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let hash_file_name = "alpha.paq";
    let dir = TempDir::new("it_outputs_file_hash_using_default_long_arg").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg("--out")
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_file_hash_using_long_arg() {
    let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let hash_file_name = "custom.paq";
    let dir = TempDir::new("it_outputs_file_hash_using_long_arg").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);
    let mut output = PathBuf::from(&source).parent().unwrap().to_path_buf();
    output.push(hash_file_name);

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg(format!("--out={}", output.as_os_str().to_str().unwrap()))
        .assert();
    assert
        .code(0)
        .stdout(format!("{expectation}\n"))
        .success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}
