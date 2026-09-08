use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn copied(hashes: &[[u8; 32]]) -> blake3::Hash {
    let mut bytes = Vec::with_capacity(hashes.len() * 32);
    for hash in hashes {
        bytes.extend_from_slice(hash);
    }
    blake3::hash(&bytes)
}

fn incremental(hashes: &[[u8; 32]]) -> blake3::Hash {
    let mut hasher = blake3::Hasher::new();
    for hash in hashes {
        hasher.update(hash);
    }
    hasher.finalize()
}

fn collected(hashes: &[[u8; 32]]) -> blake3::Hash {
    let bytes: Vec<u8> = hashes
        .iter()
        .flat_map(|hash| hash.iter().copied())
        .collect();
    blake3::hash(&bytes)
}

// Retained as a historical microbenchmark; production does not use this cast.
fn borrowed_unsafe(hashes: &[[u8; 32]]) -> blake3::Hash {
    let len = hashes.len().checked_mul(32).unwrap();
    // Arrays are contiguous without padding. The slice does not outlive hashes;
    // its pointer is non-null and aligned even for empty input.
    let bytes = unsafe { std::slice::from_raw_parts(hashes.as_ptr().cast::<u8>(), len) };
    blake3::hash(bytes)
}

fn bench_hash_list_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_final_hash");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    let mut counts = vec![0usize, 1, 10, 1_000];
    if std::env::var_os("PAQ_BENCH_LARGE").is_some() {
        counts.extend([100_000, 1_000_000]);
    }
    let methods: [(&str, fn(&[[u8; 32]]) -> blake3::Hash); 4] = [
        ("iterate_and_update", incremental),
        ("iterate_to_byte_vector", collected),
        ("iterate_to_byte_array", copied),
        ("unsafe_to_slice", borrowed_unsafe),
    ];
    for n in counts {
        let mut hashes: Vec<[u8; 32]> = (0..n)
            .map(|i| *blake3::hash(format!("test_file_{}", i % 97).as_bytes()).as_bytes())
            .collect();
        hashes.sort_unstable();
        let expected = copied(&hashes);
        group.throughput(Throughput::Bytes((n * 32) as u64));
        for (name, hash) in methods {
            assert_eq!(hash(&hashes), expected, "{name}/{n}");
            group.bench_with_input(BenchmarkId::new(name, n), &hashes, |b, values| {
                b.iter(|| black_box(hash(black_box(values)).to_hex()));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_hash_list_hash);
criterion_main!(benches);
