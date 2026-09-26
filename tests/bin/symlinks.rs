use std::path::PathBuf;

use assert_cmd::{
    cargo::cargo_bin,
    Command,
};

use crate::utils::TempDir;

#[test]
fn it_follows_links_only_when_requested() {
    let target = TempDir::new("cli_follow_links_target").unwrap();
    target.new_file("file", b"payload").unwrap();
    let dir = TempDir::new("cli_follow_links_source").unwrap();
    dir.new_symlink("file", target.path().join("file")).unwrap();
    let default_hash = paq::hash_source(dir.path(), false, false).unwrap();
    let followed_hash = paq::hash_source(target.path(), false, false).unwrap();
    assert_ne!(default_hash, followed_hash);
    Command::new(cargo_bin!("paq"))
        .arg(dir.path())
        .assert()
        .success()
        .stdout(format!("{default_hash}\n"));
    for flag in ["-L", "--follow"] {
        Command::new(cargo_bin!("paq"))
            .arg(flag)
            .arg(dir.path())
            .assert()
            .success()
            .stdout(format!("{followed_hash}\n"));
    }
}

#[test]
fn it_follows_root_directory_links_only_when_requested() {
    let target = TempDir::new("cli_follow_root_target").unwrap();
    target.new_file("file", b"payload").unwrap();
    let dir = TempDir::new("cli_follow_root_source").unwrap();
    dir.new_symlink("link", target.path().to_path_buf())
        .unwrap();
    let link = dir.path().join("link");
    let default_hash = paq::hash_source(&link, false, false).unwrap();
    let followed_hash = paq::hash_source(target.path(), false, false).unwrap();
    assert_ne!(default_hash, followed_hash);
    Command::new(cargo_bin!("paq"))
        .arg(&link)
        .assert()
        .success()
        .stdout(format!("{default_hash}\n"));
    for flag in ["-L", "--follow"] {
        Command::new(cargo_bin!("paq"))
            .arg(flag)
            .arg(&link)
            .assert()
            .success()
            .stdout(format!("{followed_hash}\n"));
    }
}

#[test]
fn it_hashes_broken_root_links_without_following() {
    let dir = TempDir::new("cli_broken_root_link").unwrap();
    dir.new_symlink("link", PathBuf::from("missing")).unwrap();
    let link = dir.path().join("link");
    let expected = paq::hash_source(&link, false, false).unwrap();
    Command::new(cargo_bin!("paq"))
        .arg(&link)
        .assert()
        .success()
        .stdout(format!("{expected}\n"));

    for flag in ["-L", "--follow"] {
        let output = dir.path().join("result.paq");
        Command::new(cargo_bin!("paq"))
            .arg(flag)
            .arg(&link)
            .arg(format!("--out={}", output.display()))
            .assert()
            .code(1)
            .stdout("");
        assert!(!output.exists());
    }
}

#[test]
fn it_writes_default_output_beside_root_links() {
    let dir = TempDir::new("cli_root_link_output").unwrap();
    std::fs::create_dir(dir.path().join("target")).unwrap();
    dir.new_file("target/file", b"payload").unwrap();

    for target in ["target", "target/file", "missing"] {
        dir.new_symlink("link", PathBuf::from(target)).unwrap();
        let link = dir.path().join("link");
        for follow in [false, true] {
            if follow && target == "missing" {
                continue;
            }
            let expected = paq::hash_source(&link, false, follow).unwrap();
            let mut command = Command::new(cargo_bin!("paq"));
            command.current_dir(dir.path()).args(["-o", "link"]);
            if follow {
                command.arg("--follow");
            }
            command.assert().success().stdout(format!("{expected}\n"));
            assert_eq!(
                std::fs::read_to_string(dir.path().join("link.paq")).unwrap(),
                format!("\"{expected}\"")
            );
            assert!(!dir.path().join(format!("{target}.paq")).exists());
            std::fs::remove_file(dir.path().join("link.paq")).unwrap();
        }
        std::fs::remove_file(link).unwrap();
    }
}

#[test]
fn it_does_not_write_a_hash_when_following_fails() {
    let dir = TempDir::new("cli_follow_links_error").unwrap();
    dir.new_symlink("link", PathBuf::from(".")).unwrap();
    let output = dir.path().join("result.paq");
    Command::new(cargo_bin!("paq"))
        .arg("-L")
        .arg(dir.path())
        .arg(format!("--out={}", output.display()))
        .assert()
        .failure()
        .stdout("");
    assert!(!output.exists());
}
