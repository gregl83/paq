use assert_cmd::{
    cargo::cargo_bin,
    Command,
};

use crate::utils::TempDir;

#[test]
fn it_returns_error_for_missing_output_directory() {
    let file_name = "alpha";
    let file_contents = "alpha-body".as_bytes();
    let dir = TempDir::new("it_returns_error_for_missing_output_directory").unwrap();
    dir.new_file(file_name, file_contents).unwrap();
    let source = dir.path().join(file_name);
    let output = dir.path().join("missing").join("alpha.paq");

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .arg(format!("--out={}", output.as_os_str().to_str().unwrap()))
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("failed to write hash to"));
}

#[test]
fn it_rejects_missing_source_arg() {
    let dir = TempDir::new("it_rejects_missing_source_arg").unwrap();
    let source = dir.path().join("missing");

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(source.as_os_str().to_str().unwrap())
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("valid file or directory path"));
}

#[cfg(target_os = "linux")]
#[test]
fn it_reports_error_for_invalid_utf8_path() {
    use std::{
        ffi::OsString,
        os::unix::ffi::OsStringExt,
    };

    let dir = TempDir::new("it_reports_error_for_invalid_utf8_path").unwrap();
    let file_name = OsString::from_vec(vec![0xff]);
    std::fs::write(dir.path().join(file_name), b"").unwrap();

    let mut cmd = Command::new(cargo_bin!("paq"));
    let assert = cmd
        .arg(dir.path().as_os_str().to_str().unwrap())
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("failed to hash"));
}
