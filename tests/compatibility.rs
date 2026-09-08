#![allow(dead_code)]
mod utils;

use std::{fs, path::Path, process::Command};
use utils::TempDir;

// Frozen format reference: explicit entry lists, independent of production's
// walker, filtering, read policy, batching and finalization implementation.
fn reference(entries: &[(&str, &[u8])]) -> String {
    let mut hashes: Vec<[u8; 32]> = entries
        .iter()
        .map(|(name, payload)| {
            let mut bytes = name.as_bytes().to_vec();
            bytes.extend_from_slice(payload);
            *blake3::hash(&bytes).as_bytes()
        })
        .collect();
    hashes.sort_unstable();
    let bytes: Vec<u8> = hashes.into_iter().flatten().collect();
    blake3::hash(&bytes).to_hex().to_string()
}

fn assert_hash(path: &Path, hidden: bool, entries: &[(&str, &[u8])]) {
    assert_eq!(
        paq::try_hash_source(path, hidden).unwrap().as_str(),
        reference(entries)
    );
}

fn command(path: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_paq"));
    command.arg(path);
    command
}

#[test]
fn root_filtering_and_duplicate_entry_digests() {
    let dir = TempDir::new("compat_root_filter").unwrap();
    let hidden = dir.path().join(".hidden");
    fs::create_dir(&hidden).unwrap();
    assert_hash(&hidden, true, &[]);
    assert_hash(&hidden, false, &[("", b"")]);
    fs::write(hidden.join("a"), b"b").unwrap();
    fs::write(hidden.join("ab"), b"").unwrap();
    fs::create_dir(hidden.join("nested")).unwrap();
    fs::write(hidden.join("nested/λ"), [0, 255, 1]).unwrap();
    let entries: &[(&str, &[u8])] = &[
        ("", b""),
        ("a", b"b"),
        ("ab", b""),
        ("nested", b""),
        ("nested/λ", &[0, 255, 1]),
    ];
    assert_hash(&hidden, false, entries);
    assert_hash(&hidden, true, &[]);
    let output = Command::new(env!("CARGO_BIN_EXE_paq"))
        .current_dir(&hidden)
        .args([".", "-i"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        format!("{}\n", reference(entries)).as_bytes()
    );
}

#[test]
fn hard_linked_names_contribute_separately() {
    let dir = TempDir::new("compat_hardlinks").unwrap();
    dir.new_file("a", b"binary\0body").unwrap();
    if let Err(error) = fs::hard_link(dir.path().join("a"), dir.path().join("b")) {
        eprintln!("SKIP hard links unavailable on this filesystem: {error}");
        return;
    }
    assert_hash(
        dir.path(),
        false,
        &[("", b""), ("a", b"binary\0body"), ("b", b"binary\0body")],
    );
}

#[test]
fn read_boundaries_preserve_root_and_nonempty_prefix_hashes() {
    let dir = TempDir::new("compat_read_boundaries").unwrap();
    let buffer = paq::FILE_BUFFER_SIZE;
    let mut sizes = vec![
        0,
        1,
        63,
        64,
        65,
        1_023,
        1_024,
        1_025,
        buffer - 1,
        buffer,
        buffer + 1,
    ];
    #[cfg(not(target_os = "windows"))]
    sizes.extend([1_048_575, 1_048_576, 1_048_577]);
    let file = dir.path().join("nonempty-prefix-λ");
    for size in sizes {
        let content: Vec<u8> = (0..size).map(|i| ((i * 73 + 19) % 256) as u8).collect();
        fs::write(&file, &content).unwrap();
        assert_hash(&file, false, &[("", &content)]);
        assert_hash(
            dir.path(),
            false,
            &[("", b""), ("nonempty-prefix-λ", &content)],
        );
    }
}

#[test]
fn batch_boundaries_preserve_hashes_in_configured_pools() {
    for count in [0, 1, 98, 99, 100, 101, 199, 200] {
        let dir = TempDir::new(&format!("compat_batches_{count}")).unwrap();
        let names: Vec<String> = (0..count).map(|i| format!("{i:04}")).collect();
        for name in &names {
            dir.new_file(name, b"alpha-body").unwrap();
        }
        dir.new_file(".hidden", b"hidden-body").unwrap();
        let mut entries: Vec<(&str, &[u8])> = vec![("", b"")];
        entries.extend(
            names
                .iter()
                .map(|name| (name.as_str(), b"alpha-body".as_slice())),
        );
        for hidden in [true, false] {
            if !hidden {
                entries.push((".hidden", b"hidden-body"));
            }
            let expected = reference(&entries);
            for threads in [1, 2, 4] {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                let actual = pool.install(|| paq::try_hash_source(dir.path(), hidden).unwrap());
                assert_eq!(
                    actual.as_str(),
                    expected,
                    "files={count}, threads={threads}, hidden={hidden}"
                );
            }
            assert_eq!(
                paq::try_hash_source(dir.path(), hidden).unwrap().as_str(),
                expected
            );
        }
    }
}

#[test]
fn missing_source_preserves_walk_path_and_panic_wrapper() {
    let dir = TempDir::new("compat_missing").unwrap();
    let source = dir.path().join("missing");
    match paq::try_hash_source(&source, false).unwrap_err() {
        paq::Error::Walk(error) => assert_eq!(error.path(), Some(source.as_path())),
        error => panic!("unexpected error variant: {error:?}"),
    }
    assert!(std::panic::catch_unwind(|| paq::hash_source(&source, false)).is_err());
    let output = command(&source).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn output_inside_source_is_written_after_hashing_and_has_exact_json_bytes() {
    let dir = TempDir::new("compat_output_order").unwrap();
    dir.new_file("file", b"alpha-body").unwrap();
    let output_path = dir.path().join("result.paq");
    for old_output in [None, Some(b"old output bytes".as_slice())] {
        if output_path.exists() {
            fs::remove_file(&output_path).unwrap();
        }
        let mut entries: Vec<(&str, &[u8])> = vec![("", b""), ("file", b"alpha-body")];
        if let Some(bytes) = old_output {
            fs::write(&output_path, bytes).unwrap();
            entries.push(("result.paq", bytes));
        }
        let expected = reference(&entries);
        let output = command(dir.path())
            .arg(format!("--out={}", output_path.display()))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
        assert!(output.stderr.is_empty());
        assert_eq!(
            fs::read(&output_path).unwrap(),
            format!("\"{expected}\"").as_bytes()
        );
    }
    let output = command(dir.path())
        .arg(format!("-o={}/missing/result.paq", dir.path().display()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        ffi::OsString,
        os::unix::{
            ffi::OsStringExt,
            fs::{symlink, PermissionsExt},
        },
    };

    #[test]
    fn root_and_interior_symlinks_preserve_entry_sets() {
        let dir = TempDir::new("compat_symlinks").unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("file"), b"target-body").unwrap();
        for name in ["link", ".link"] {
            symlink("target", dir.path().join(name)).unwrap();
            assert_hash(
                &dir.path().join(name),
                false,
                &[("", b"target"), ("file", b"target-body")],
            );
        }
        assert_hash(&dir.path().join(".link"), true, &[("file", b"target-body")]);
        symlink("target/file", dir.path().join("file-link")).unwrap();
        assert_hash(
            &dir.path().join("file-link"),
            false,
            &[("", b"target/file")],
        );
        symlink("absent", dir.path().join("broken")).unwrap();
        assert!(matches!(
            paq::try_hash_source(&dir.path().join("broken"), false),
            Err(paq::Error::Walk(_))
        ));
        assert_hash(
            dir.path(),
            false,
            &[
                ("", b""),
                ("target", b""),
                ("target/file", b"target-body"),
                ("link", b"target"),
                (".link", b"target"),
                ("file-link", b"target/file"),
                ("broken", b"absent"),
            ],
        );
    }

    #[test]
    fn invalid_utf8_root_descendant_and_target_have_distinct_behavior() {
        let dir = TempDir::new("compat_invalid_utf8").unwrap();
        let name = OsString::from_vec(vec![b'.', 255]);
        let path = dir.path().join(&name);
        fs::write(&path, b"body").unwrap();
        // Only the relative path is validated; an invalid root basename is allowed.
        for hidden in [false, true] {
            assert_hash(&path, hidden, &[("", b"body")]);
            assert!(matches!(paq::try_hash_source(dir.path(), hidden),
                Err(paq::Error::InvalidUtf8Path(error_path)) if error_path == path));
        }
        fs::remove_file(&path).unwrap();
        let target = OsString::from_vec(vec![255]);
        symlink(&target, dir.path().join("link")).unwrap();
        assert!(matches!(paq::try_hash_source(dir.path(), false),
            Err(paq::Error::InvalidUtf8Path(error_path)) if error_path == Path::new(&target)));
    }

    #[test]
    fn unreadable_empty_files_remain_path_only_and_nonempty_files_return_io() {
        let dir = TempDir::new("compat_permissions_file").unwrap();
        let file = dir.path().join("file");
        fs::write(&file, b"").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0)).unwrap();
        let readable = fs::File::open(&file).is_ok();
        let empty_result = paq::try_hash_source(dir.path(), false);
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        if readable {
            eprintln!("SKIP file permission assertions: host can read mode-000 files");
            return;
        }
        assert_eq!(
            empty_result.unwrap().as_str(),
            reference(&[("", b""), ("file", b"")])
        );
        fs::write(&file, b"nonempty").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0)).unwrap();
        let result = paq::try_hash_source(dir.path(), false);
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(matches!(result, Err(paq::Error::Io { path, source })
            if path == file && source.kind() == std::io::ErrorKind::PermissionDenied));
    }

    #[test]
    fn inaccessible_directory_returns_walk_error_with_path() {
        let dir = TempDir::new("compat_permissions_dir").unwrap();
        let child = dir.path().join("child");
        fs::create_dir(&child).unwrap();
        fs::set_permissions(&child, fs::Permissions::from_mode(0)).unwrap();
        let readable = fs::read_dir(&child).is_ok();
        let result = paq::try_hash_source(dir.path(), false);
        fs::set_permissions(&child, fs::Permissions::from_mode(0o700)).unwrap();
        if readable {
            eprintln!(
                "SKIP directory permission assertions: host can enumerate mode-000 directories"
            );
            return;
        }
        assert!(
            matches!(result, Err(paq::Error::Walk(error)) if error.path() == Some(child.as_path()))
        );
    }

    #[test]
    fn special_entries_contribute_their_path_without_reading() {
        let dir = TempDir::new("compat_special").unwrap();
        let socket = dir.path().join("socket");
        let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        assert_hash(dir.path(), false, &[("", b""), ("socket", b"")]);
    }
}

#[cfg(windows)]
#[test]
fn windows_root_links_preserve_target_text_and_descendants() {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    let dir = TempDir::new("compat_windows_links").unwrap();
    let target = dir.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("file"), b"target-body").unwrap();
    if let Err(error) = symlink_dir("target", dir.path().join("link")) {
        if matches!(
            error.kind(),
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
        ) {
            eprintln!("SKIP Windows symlinks unavailable: {error}");
            return;
        }
        panic!("failed to create test symlink: {error}");
    }
    assert_hash(
        &dir.path().join("link"),
        false,
        &[("", b"target"), ("file", b"target-body")],
    );
    symlink_dir("target", dir.path().join(".link")).unwrap();
    assert_hash(&dir.path().join(".link"), true, &[("file", b"target-body")]);
    symlink_file(r"target\file", dir.path().join("file-link")).unwrap();
    assert_hash(
        &dir.path().join("file-link"),
        false,
        &[("", b"target/file")],
    );
    symlink_file("absent", dir.path().join("broken")).unwrap();
    assert!(matches!(
        paq::try_hash_source(&dir.path().join("broken"), false),
        Err(paq::Error::Walk(_))
    ));
}
