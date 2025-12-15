#[path="../src/lib.rs"]
mod paq;
mod utils;

use std::{
    hint::black_box,
    time::Duration,
};

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};

use utils::TempDir;


fn bench_paq_library(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "hash_source"
    );
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    let dir = TempDir::new(
        "bench_hashes_directory_files"
    ).unwrap();

    for i in 0..100 {
        dir.new_file(
            format!("{i}").as_str(),
            format!("{i}-body").as_bytes()
        ).unwrap()
    }

    let source = dir.path().canonicalize().unwrap();

    group.bench_with_input(
        "hashes_directory_with_files",
        &source,
        |b, source| {
            b.iter(|| paq::hash_source(
                black_box(source),
                false
            ))
        },
    );

    group.finish();
}

criterion_group!(benches, bench_paq_library);
criterion_main!(benches);
