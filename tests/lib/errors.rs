use crate::utils::TempDir;

#[test]
fn it_returns_error_for_missing_source() {
    let dir = TempDir::new("it_returns_error_for_missing_source").unwrap();
    let source = dir.path().join("missing");

    let error = paq::try_hash_source(&source, true).unwrap_err();
    assert!(error
        .to_string()
        .starts_with("failed to traverse source:"));
    assert!(matches!(error, paq::Error::Walk(_)));
}

#[cfg(target_os = "linux")]
#[test]
fn it_returns_error_for_invalid_utf8_path() {
    use std::{
        ffi::OsString,
        os::unix::ffi::OsStringExt,
    };

    let dir = TempDir::new("it_returns_error_for_invalid_utf8_path").unwrap();
    let file_name = OsString::from_vec(vec![0xff]);
    std::fs::write(dir.path().join(file_name), b"").unwrap();

    let error = paq::try_hash_source(dir.path(), false).unwrap_err();
    assert!(matches!(error, paq::Error::InvalidUtf8Path(_)));
}
