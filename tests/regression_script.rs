#![cfg(unix)]
#![allow(dead_code)]
mod utils;

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
use utils::TempDir;

fn executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn regression_preflight_checks_exact_bytes_exit_status_and_quoting() {
    let dir = TempDir::new("regression-script").unwrap();
    let tools = dir.path().join("tools");
    let temp = dir.path().join("temporary");
    fs::create_dir(&tools).unwrap();
    fs::create_dir(&temp).unwrap();
    let before = dir.path().join("before ' $() ; executable");
    let after = dir.path().join("after `touch INJECTED` executable");
    let source = "-source ' $(touch INJECTED) ;\nwith newline";
    fs::create_dir(dir.path().join(source)).unwrap();
    let marker = dir.path().join("hyperfine-called");
    executable(
        &tools.join("hyperfine"),
        r#"#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == --shell && "$2" == bash && "$3" == --warmup && "$4" == 3 ]]
[[ "$5" == -n && "$8" == -n && "$#" == 10 ]]
bash -c "$7" >/dev/null
bash -c "${10}" >/dev/null
: > "$MARKER"
"#,
    );
    let good = r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "$EXPECT_HIDDEN" == yes ]]; then
  [[ "$1" == --ignore-hidden || "$1" == -i ]]
  shift
fi
[[ "$#" == 2 && "$1" == -- && "$2" == "$EXPECTED_SOURCE" ]]
printf 'same hash\n'
"#;
    executable(&before, good);
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("bin/regression.sh");
    let path = std::env::join_paths(
        std::iter::once(tools).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    for (candidate, flag, success) in [
        (good, None, true),
        (good, Some("--ignore-hidden"), true),
        (good, Some("-i"), true),
        (
            "#!/usr/bin/env bash\nprintf 'different hash\\n'\n",
            None,
            false,
        ),
        ("#!/usr/bin/env bash\nprintf 'same hash'\n", None, false),
        (
            "#!/usr/bin/env bash\nprintf 'same hash\\n'\nexit 7\n",
            None,
            false,
        ),
    ] {
        executable(&after, candidate);
        if marker.exists() {
            fs::remove_file(&marker).unwrap();
        }
        let mut cmd = Command::new("bash");
        cmd.arg(&script)
            .arg(&before)
            .arg(&after)
            .arg(source)
            .current_dir(dir.path())
            .env("PATH", &path)
            .env("TMPDIR", &temp)
            .env("EXPECTED_SOURCE", source)
            .env("EXPECT_HIDDEN", if flag.is_some() { "yes" } else { "no" })
            .env("MARKER", &marker);
        if let Some(flag) = flag {
            cmd.arg(flag);
        }
        let output = cmd.output().unwrap();
        assert_eq!(
            output.status.success(),
            success,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            marker.exists(),
            success,
            "only compatible executions may reach timing"
        );
        assert_eq!(
            fs::read_dir(&temp).unwrap().count(),
            0,
            "preflight must clean its files"
        );
        assert!(!dir.path().join("INJECTED").exists());
    }
}
