mod utils;

mod lib {
    use crate::utils::TempDir;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn it_hashes_single_file() {
        let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

        let file_name = "alpha";
        let file_contents = "alpha-body".as_bytes();
        let dir = TempDir::new("it_hashes_single_file").unwrap();
        dir.new_file(file_name, file_contents).unwrap();
        let source = dir.path().join(file_name);

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_not_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_not_ignored[..], expectation);
    }

    #[test]
    fn it_hashes_directory() {
        let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

        let dir = TempDir::new("it_hashes_directory").unwrap();
        let source = dir.path().canonicalize().unwrap();

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_not_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_not_ignored[..], expectation);
    }

    #[test]
    fn it_hashes_directory_from_any_path() {
        let expectation = "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

        let dir = TempDir::new("it_hashes_directory_from_any_path").unwrap();
        let source = dir.path().canonicalize().unwrap();
        let original_path = env::current_dir().unwrap();
        let new_path = PathBuf::from("/");

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_ignored[..], expectation);

        env::set_current_dir(new_path).unwrap();

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_ignored[..], expectation);

        env::set_current_dir(original_path).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn it_hashes_directory_relative_symlink_without_following() {
        let expectation = "5bb837eff87dee38d63c081bc30a8d0ce7cc871c8b32e38e3e40f9ccdef4db98";

        let symlink_name = "symlink";
        let symlink_target = PathBuf::from("target");
        let dir = TempDir::new("it_hashes_directory_relative_symlink_without_following").unwrap();
        dir.new_symlink(symlink_name, symlink_target).unwrap();
        let source = dir.path().canonicalize().unwrap();

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_ignored[..], expectation);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn it_hashes_directory_absolute_symlink_without_following() {
        let expectation = "60fbb028703074fe8b17a20155696f6dffab6a3de071b716792350f946d917aa";

        let symlink_name = "symlink";
        let symlink_target = PathBuf::from("/");
        let dir = TempDir::new("it_hashes_directory_absolute_symlink_without_following").unwrap();
        dir.new_symlink(symlink_name, symlink_target).unwrap();
        let source = dir.path().canonicalize().unwrap();

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_ignored[..], expectation);
    }

    #[test]
    fn it_hashes_directory_with_file() {
        let expectation = "7ed5febd35e277763cdfc3e4bee136acf38e48e9462972a732cc4d348a37d653";

        let file_name = "alpha";
        let file_contents = "alpha-body".as_bytes();
        let dir = TempDir::new("it_hashes_directory_with_file").unwrap();
        dir.new_file(file_name, file_contents).unwrap();
        let source = dir.path().canonicalize().unwrap();

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation);
        let hash_not_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_not_ignored[..], expectation);
    }

    #[test]
    fn it_hashes_directory_with_ignored_file() {
        let expectation_not_ignored =
            "e383192a5ef45576817b4222e455e3d538ae3bab279a62c0a8b67279ad007072";
        let expectation_ignored =
            "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

        let file_name = ".ignored";
        let file_contents = ".ignored-body".as_bytes();
        let dir = TempDir::new("it_hashes_directory_with_ignored_file").unwrap();
        dir.new_file(file_name, file_contents).unwrap();
        let source = dir.path().canonicalize().unwrap();

        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation_ignored);
        let hash_not_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_not_ignored[..], expectation_not_ignored);
    }

    #[test]
    fn it_hashes_directory_with_ignored_subdirectory() {
        let expectation_not_ignored =
            "f38a56a87aca98131b2fa5914fd13bc11f5823602293e8d84b5c69000b33ebf2";
        let expectation_ignored =
            "82878ed8a480ee41775636820e05a934ca5c747223ca64306658ee5982e6c227";

        let dir = TempDir::new("it_hashes_directory_with_ignored_subdirectory").unwrap();
        let source = dir.path().canonicalize().unwrap();

        let subdir = TempDir::new("it_hashes_directory_with_ignored_subdirectory/.test").unwrap();
        let hash_ignored = paq::hash_source(&source, true);
        assert_eq!(&hash_ignored[..], expectation_ignored);
        let hash_not_ignored = paq::hash_source(&source, false);
        assert_eq!(&hash_not_ignored[..], expectation_not_ignored);

        println!(
            "prevent early subdir drop for: {}",
            subdir.path().as_os_str().to_str().unwrap()
        )
    }

    #[test]
    fn it_hashes_directory_files_consistently() {
        let expectation = "59a0db8e557830ccb77ac0e4556931925cdc592a1a8b83e1bdc3c8da406f4ef5";

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
        dir.new_file(charlie_file_name, charlie_file_contents)
            .unwrap();
        dir.new_file(one_file_name, one_file_contents).unwrap();
        dir.new_file(nine_file_name, nine_file_contents).unwrap();
        let source = dir.path().canonicalize().unwrap();

        for _ in 0..50 {
            let hash_ignored = paq::hash_source(&source, true);
            assert_eq!(&hash_ignored[..], expectation);
            let hash_not_ignored = paq::hash_source(&source, false);
            assert_eq!(&hash_not_ignored[..], expectation);
        }
    }
}

mod bin {
    use crate::utils::TempDir;
    use assert_cmd::Command;
    use std::path::PathBuf;

    #[test]
    fn it_outputs_file_hash_using_default_short_arg() {
        let expectation = "48ec422c86fd2aa1ac182f832c10cf6cb07e4b89d88b83a7794bd8773460072c";

        let file_name = "alpha";
        let file_contents = "alpha-body".as_bytes();
        let hash_file_name = "alpha.paq";
        let dir = TempDir::new("it_outputs_file_hash_using_default_short_arg").unwrap();
        dir.new_file(file_name, file_contents).unwrap();
        let source = dir.path().join(file_name);

        let mut cmd = Command::cargo_bin("paq").unwrap();
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

        let mut cmd = Command::cargo_bin("paq").unwrap();
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

        let mut cmd = Command::cargo_bin("paq").unwrap();
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

        let mut cmd = Command::cargo_bin("paq").unwrap();
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
}
