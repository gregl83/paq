use std::path::PathBuf;

use crate::utils::TempDir;

#[cfg(target_family = "unix")]
#[test]
fn it_hashes_directory_relative_symlink_without_following() {
    let expectation = "e2c989a91166ea125a275abc9832cef4ffd21953a5eeb31763f850a7aa7c7916";

    let symlink_name = "symlink";
    let symlink_target = PathBuf::from("target");
    let dir = TempDir::new("it_hashes_directory_relative_symlink_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true, false).unwrap();
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(&source, false, false).unwrap();
    assert_eq!(&hash_ignored[..], expectation);
}

#[cfg(target_family = "unix")]
#[test]
fn it_hashes_directory_absolute_symlink_without_following() {
    let expectation = "bef6e261b38ff07d099269e1783808888b85844c08c37cfff42722e1242bd7af";

    let symlink_name = "symlink";
    let symlink_target = PathBuf::from("/");
    let dir = TempDir::new("it_hashes_directory_absolute_symlink_without_following").unwrap();
    dir.new_symlink(symlink_name, symlink_target).unwrap();
    let source = dir.path().canonicalize().unwrap();

    let hash_ignored = paq::hash_source(&source, true, false).unwrap();
    assert_eq!(&hash_ignored[..], expectation);
    let hash_ignored = paq::hash_source(&source, false, false).unwrap();
    assert_eq!(&hash_ignored[..], expectation);
}

#[test]
fn it_follows_root_links_only_when_requested() {
    let dir = TempDir::new("follow_root_links").unwrap();
    std::fs::create_dir(dir.path().join("directory")).unwrap();
    dir.new_file("directory/file", b"payload").unwrap();

    for target in ["directory", "directory/file", "missing"] {
        let link = dir.path().join("link");
        dir.new_symlink("link", PathBuf::from(target)).unwrap();
        // A root link has an empty relative path, symlink type, and target text.
        let mut entry = blake3::Hasher::new();
        entry.update(&[0, 3]);
        entry.update(target.as_bytes());
        let expected = blake3::hash(entry.finalize().as_bytes()).to_hex();
        assert_eq!(paq::hash_source(&link, false, false).unwrap(), expected);

        if target == "missing" {
            assert!(matches!(
                paq::hash_source(&link, false, true),
                Err(paq::Error::Walk(_))
            ));
        } else {
            assert_eq!(
                paq::hash_source(&link, false, true).unwrap(),
                paq::hash_source(&dir.path().join(target), false, false).unwrap()
            );
        }
        std::fs::remove_file(link).unwrap();
    }
}

#[test]
fn it_follows_targets_under_the_link_path() {
    let target = TempDir::new("follow_links_external_target").unwrap();
    target.new_file("file", b"payload").unwrap();
    target.new_file(".hidden", b"hidden").unwrap();
    let linked = TempDir::new("follow_links_source").unwrap();
    linked
        .new_symlink("directory", target.path().to_path_buf())
        .unwrap();
    linked
        .new_symlink("file", target.path().join("file"))
        .unwrap();
    let copied = TempDir::new("follow_links_reference").unwrap();
    std::fs::create_dir(copied.path().join("directory")).unwrap();
    copied.new_file("directory/file", b"payload").unwrap();
    copied.new_file("directory/.hidden", b"hidden").unwrap();
    copied.new_file("file", b"payload").unwrap();

    for ignore_hidden in [false, true] {
        assert_eq!(
            paq::hash_source(linked.path(), ignore_hidden, true).unwrap(),
            paq::hash_source(copied.path(), ignore_hidden, false).unwrap()
        );
        assert_ne!(
            paq::hash_source(linked.path(), ignore_hidden, false).unwrap(),
            paq::hash_source(copied.path(), ignore_hidden, false).unwrap()
        );
    }
    // Root file and directory links also resolve when explicitly following.
    for name in ["file", "directory"] {
        assert_eq!(
            paq::hash_source(&linked.path().join(name), false, true).unwrap(),
            paq::hash_source(&copied.path().join(name), false, false).unwrap()
        );
    }
}

#[test]
fn it_reports_broken_links_and_cycles_when_following() {
    for (name, target) in [("broken", "missing"), ("cycle", ".")] {
        let dir = TempDir::new(&format!("follow_links_error_{name}")).unwrap();
        dir.new_symlink("link", PathBuf::from(target)).unwrap();
        assert!(paq::hash_source(dir.path(), false, false).is_ok());
        assert!(matches!(
            paq::hash_source(dir.path(), false, true),
            Err(paq::Error::Walk(_))
        ));
    }
}

#[test]
fn it_prunes_hidden_linked_directories() {
    let target = TempDir::new("follow_hidden_target").unwrap();
    target.new_file("file", b"payload").unwrap();
    let dir = TempDir::new("follow_hidden_source").unwrap();
    dir.new_symlink(".hidden", target.path().to_path_buf())
        .unwrap();
    let empty = TempDir::new("follow_hidden_reference").unwrap();
    assert_eq!(
        paq::hash_source(dir.path(), true, true).unwrap(),
        paq::hash_source(empty.path(), true, false).unwrap()
    );
}
