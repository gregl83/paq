use std::{
    hint::black_box,
    time::Duration,
};

use criterion::{
    criterion_group,
    criterion_main,
    BatchSize,
    BenchmarkId,
    Criterion,
    Throughput,
};
use rayon::prelude::ParallelSliceMut;

fn bench_hash_list_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_blake3_hashes");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    // rayon warmup (ensure thread pool init)
    let mut warmup: Vec<i32> = (0..64).collect();
    warmup.par_sort_unstable();

    // run benchmarks with various hash list sizes
    for &n in &[1_000usize, 2_000, 5_000] {
        // generate hash list
        let file_hashes: Vec<[u8; 32]> = (0..n)
            .map(|i| {
                blake3::hash(format!("test_file_{i}").as_bytes())
                    .as_bytes()
                    .to_owned()
            })
            .collect();

        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("sequential", n), &file_hashes, |b, src| {
            b.iter_batched_ref(
                || src.clone(),
                |v| {
                    v.sort_unstable();
                    black_box(v);
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("parallel", n), &file_hashes, |b, src| {
            b.iter_batched_ref(
                || src.clone(),
                |v| {
                    v.par_sort_unstable();
                    black_box(v);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_hash_list_sort);
criterion_main!(benches);
