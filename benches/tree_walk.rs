mod utils;

use std::{
    fs,
    hint::black_box,
    path::Path,
    time::Duration,
};

use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
    Throughput,
};
use utils::TempDir;

fn entry_kind(kind: fs::FileType) -> u8 {
    if kind.is_file() {
        1
    } else if kind.is_dir() {
        2
    } else if kind.is_symlink() {
        3
    } else {
        4
    }
}

fn walkdir(root: &Path) -> walkdir::IntoIter {
    // Include hidden entries and do not follow interior symlinks in either walker.
    walkdir::WalkDir::new(root).follow_links(false).into_iter()
}

fn jwalk(root: &Path) -> jwalk::WalkDir {
    jwalk::WalkDir::new(root)
        .skip_hidden(false)
        .follow_links(false)
        .sort(false)
}

fn fixture(shape: &str) -> TempDir {
    let dir = TempDir::new(&format!("bench_tree_walk_{shape}_{}", std::process::id())).unwrap();
    let root = dir.path();
    match shape {
        "flat" => {
            for i in 0..1_000 {
                fs::write(root.join(format!("file-{i}")), b"").unwrap();
            }
        }
        "deep" => {
            let mut path = root.to_path_buf();
            // Short names keep the fixture usable with Windows path limits.
            for _ in 0..32 {
                path.push("d");
                fs::create_dir(&path).unwrap();
                for i in 0..32 {
                    fs::write(path.join(format!("f{i}")), b"").unwrap();
                }
            }
        }
        "wide" => {
            for i in 0..100 {
                let path = root.join(format!("dir-{i}"));
                fs::create_dir(&path).unwrap();
                for j in 0..10 {
                    fs::write(path.join(format!("file-{j}")), b"").unwrap();
                }
            }
        }
        _ => unreachable!(),
    }
    fs::create_dir(root.join(".hidden")).unwrap();
    fs::write(root.join(".hidden/file"), b"").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(".hidden", root.join("directory-link")).unwrap();
        std::os::unix::fs::symlink("missing", root.join("broken-link")).unwrap();
    }
    dir
}

fn validate(root: &Path) -> u64 {
    // Collect and sort only for validation, never inside the timed traversal.
    let mut sequential: Vec<_> = walkdir(root)
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.path().strip_prefix(root).unwrap().to_path_buf(),
                entry_kind(entry.file_type()),
            )
        })
        .collect();
    let mut parallel: Vec<_> = jwalk(root)
        .into_iter()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.path().strip_prefix(root).unwrap().to_path_buf(),
                entry_kind(entry.file_type()),
            )
        })
        .collect();
    sequential.sort_unstable();
    parallel.sort_unstable();
    assert_eq!(
        sequential, parallel,
        "walkers must return identical entries"
    );
    sequential.len() as u64
}

fn bench_tree_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_walk");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    for shape in ["flat", "deep", "wide"] {
        let dir = fixture(shape);
        let root = dir.path();
        // Also warms jwalk's thread pool and filesystem caches before timing.
        let entries = validate(root);
        group.throughput(Throughput::Elements(entries));

        group.bench_with_input(BenchmarkId::new("walkdir", shape), &root, |b, root| {
            b.iter(|| {
                let mut count = 0usize;
                for entry in walkdir(black_box(root)) {
                    let entry = entry.unwrap();
                    black_box((entry.file_name(), entry.file_type()));
                    count += 1;
                }
                black_box(count)
            });
        });
        group.bench_with_input(BenchmarkId::new("jwalk", shape), &root, |b, root| {
            b.iter(|| {
                let mut count = 0usize;
                for entry in jwalk(black_box(root)) {
                    let entry = entry.unwrap();
                    black_box((entry.file_name(), entry.file_type()));
                    count += 1;
                }
                black_box(count)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tree_walk);
criterion_main!(benches);
