use std::path::PathBuf;

use assert_cmd::{
    cargo::cargo_bin,
    Command,
};

use crate::utils::TempDir;

#[test]
fn it_outputs_directory_hash_using_default_source() {
    let expectation = "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

    let dir = TempDir::new("it_outputs_directory_hash_using_default_source").unwrap();

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd.current_dir(dir.path()).assert();
    assert.code(0).stdout(format!("{expectation}\n")).success();
}

#[test]
fn it_outputs_file_hash_without_output() {
    let expectation = "31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e";

    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let hash_file_name = "alpha.paq";
    let dir = TempDir::new("it_outputs_file_hash_without_output").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd.arg(source.as_os_str().to_str().unwrap()).assert();
    assert.code(0).stdout(format!("{expectation}\n")).success();

    assert!(!dir.path().join(hash_file_name).exists());
}

#[test]
fn it_outputs_file_hash_using_default_short_arg() {
    let expectation = "31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_directory_hash_using_default_short_arg() {
    let expectation = "7887758062e01b93b78eedf93fcfa49c3d0952ee3a278ace176e3dab4d50084e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_file_hash_using_short_arg() {
    let expectation = "31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_file_hash_using_default_long_arg() {
    let expectation = "31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}

#[test]
fn it_outputs_file_hash_using_long_arg() {
    let expectation = "31611f66817b666bccba70178e3bee75d23ed12fffc9bd30e98e1b912e73194e";

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
    assert.code(0).stdout(format!("{expectation}\n")).success();

    let file_hash = dir.read_file(hash_file_name).unwrap();
    assert_eq!(
        file_hash.as_slice(),
        format!("\"{expectation}\"").as_bytes()
    );
}
