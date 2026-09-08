mod utils;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::{fs, hint::black_box, path::Path, time::Duration};
use utils::TempDir;

fn measure(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    path: &Path,
) {
    for ignore_hidden in [false, true] {
        let expected = paq::try_hash_source(path, ignore_hidden).unwrap();
        assert_eq!(paq::try_hash_source(path, ignore_hidden).unwrap(), expected);
        group.bench_with_input(
            BenchmarkId::new(
                name,
                if ignore_hidden {
                    "hidden_filtered"
                } else {
                    "all"
                },
            ),
            &ignore_hidden,
            |b, ignore_hidden| {
                b.iter(|| black_box(paq::try_hash_source(black_box(path), *ignore_hidden).unwrap()))
            },
        );
    }
}

fn bench_paq_library(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_source");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    // Warmed-library measurements exclude global thread-pool initialization.
    black_box(rayon::current_num_threads());
    let dir = TempDir::new("functional_empty").unwrap();
    measure(&mut group, "empty", dir.path());
    dir.new_file("single", b"alpha-body").unwrap();
    measure(&mut group, "single_file", &dir.path().join("single"));
    for count in [98, 99, 100, 101, 199, 200, 1_000] {
        let dir = TempDir::new(&format!("functional_tiny_{count}")).unwrap();
        for i in 0..count {
            dir.new_file(&format!("{i:06}"), format!("{i}-body").as_bytes())
                .unwrap();
        }
        measure(&mut group, &format!("tiny_{count}_files"), dir.path());
    }
    let mixed = TempDir::new("functional_mixed").unwrap();
    for i in 0..128 {
        let size = [0, 10, 1_024, 1_025, 32_768, 1_048_576][i % 6];
        mixed
            .new_file_with_random_data(&format!("{i:06}"), size)
            .unwrap();
    }
    mixed.new_file(".hidden", b"hidden-body").unwrap();
    measure(&mut group, "mixed_128", mixed.path());
    for (name, depth, width) in [("deep", 40, 1), ("wide", 1, 256)] {
        let dir = TempDir::new(&format!("functional_{name}")).unwrap();
        let mut parent = dir.path().to_path_buf();
        for _ in 0..depth {
            for i in 0..width {
                let child = parent.join(format!("d{i}"));
                fs::create_dir(&child).unwrap();
                fs::write(child.join("file"), b"tree-body").unwrap();
            }
            parent.push("d0");
        }
        measure(&mut group, name, dir.path());
    }
    if std::env::var_os("PAQ_BENCH_LARGE").is_some() {
        let clustered = TempDir::new("functional_clustered").unwrap();
        for i in 0..8 {
            clustered
                .new_file_with_random_data(&format!("{i:06}"), 8 * 1_048_576)
                .unwrap();
        }
        measure(&mut group, "clustered_8x8mib", clustered.path());
        let tiny = TempDir::new("functional_many_tiny").unwrap();
        for i in 0..10_000 {
            tiny.new_file(&format!("{i:06}"), b"alpha-body").unwrap();
        }
        measure(&mut group, "tiny_10000_files", tiny.path());
    }
    // Caller supplies the pinned go1.25.0 checkout with its .git removed.
    if let Some(path) = std::env::var_os("PAQ_BENCH_GO") {
        let path = Path::new(&path);
        assert!(
            !path.join(".git").exists(),
            "remove Go's .git before measuring"
        );
        measure(&mut group, "go1.25.0", path);
    }
    group.finish();
}

criterion_group!(benches, bench_paq_library);
criterion_main!(benches);
