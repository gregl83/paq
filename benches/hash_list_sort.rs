use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use rayon::prelude::*;

fn bench_hash_list_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_blake3_hashes");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    // Explicitly initialize the pool; a tiny sort can skip parallel execution.
    black_box(rayon::current_num_threads());
    let mut counts = vec![0usize, 1, 100, 1_000, 2_000, 5_000];
    if std::env::var_os("PAQ_BENCH_LARGE").is_some() {
        counts.extend([100_000, 1_000_000]);
    }
    for n in counts {
        let random: Vec<[u8; 32]> = (0..n)
            .map(|i| *blake3::hash(format!("test_file_{i}").as_bytes()).as_bytes())
            .collect();
        for distribution in ["random", "sorted", "reverse", "duplicates"] {
            let mut source = random.clone();
            match distribution {
                "sorted" => source.sort_unstable(),
                "reverse" => source.sort_unstable_by(|a, b| b.cmp(a)),
                "duplicates" => source.iter_mut().enumerate().for_each(|(i, v)| {
                    *v = *blake3::hash(&[(i % 8) as u8]).as_bytes();
                }),
                _ => {}
            }
            let mut expected = source.clone();
            expected.sort_unstable();
            let mut parallel = source.clone();
            parallel.par_sort_unstable();
            assert_eq!(parallel, expected, "ordering and multiplicity");
            group.throughput(Throughput::Elements(n as u64));
            for method in ["sequential", "parallel"] {
                group.bench_with_input(
                    BenchmarkId::new(format!("{method}/{distribution}"), n),
                    &source,
                    |b, source| {
                        b.iter_batched(
                            || source.clone(),
                            |mut values| {
                                if method == "parallel" {
                                    values.par_sort_unstable();
                                } else {
                                    values.sort_unstable();
                                }
                                black_box(values)
                            },
                            // Never retain many million-entry clones at once.
                            BatchSize::PerIteration,
                        );
                    },
                );
            }
        }
    }
    group.finish();
}

criterion_group!(benches, bench_hash_list_sort);
criterion_main!(benches);
