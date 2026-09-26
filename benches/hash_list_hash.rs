use std::{
    hint::black_box,
    time::Duration,
};

use blake3::{
    Hash,
    Hasher,
};
use criterion::{
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
    Throughput,
};

fn iterate_and_update(hashes: &[[u8; 32]]) -> Hash {
    let mut hasher = Hasher::new();
    for hash in hashes {
        hasher.update(hash);
    }
    hasher.finalize()
}

fn iterate_to_byte_vector(hashes: &[[u8; 32]]) -> Hash {
    let bytes: Vec<u8> = hashes
        .iter()
        .flat_map(|hash| hash.iter())
        .copied()
        .collect();
    blake3::hash(&bytes)
}

// Matches the allocation and copying in the current get_hashes_root.
fn copy_to_vector(hashes: &[[u8; 32]]) -> Hash {
    let mut bytes = Vec::with_capacity(hashes.len() * 32);
    for hash in hashes {
        bytes.extend_from_slice(hash);
    }
    blake3::hash(&bytes)
}

// F04: borrow the same contiguous bytes without allocation or copying.
fn borrow_as_slice(hashes: &[[u8; 32]]) -> Hash {
    blake3::hash(hashes.as_flattened())
}

fn bench_hash_list_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_final_hash");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    // Empty/single inputs, small trees, a Go-sized tree, and a large tree.
    for count in [0usize, 1, 100, 1_000, 15_000, 200_000] {
        let mut hashes: Vec<[u8; 32]> = (0..count)
            // Include duplicates to check that every digest is retained.
            .map(|i| *blake3::hash(format!("file-{}", i / 2).as_bytes()).as_bytes())
            .collect();
        hashes.sort_unstable();
        let expected = iterate_and_update(&hashes);
        group.throughput(Throughput::Elements(count as u64));

        macro_rules! bench_method {
            ($method:ident) => {
                assert_eq!($method(&hashes), expected);
                group.bench_with_input(
                    BenchmarkId::new(stringify!($method), count),
                    &hashes,
                    |b, hashes| {
                        // Fixture generation/sorting is excluded; each copy variant's
                        // allocation, copying and release remain inside timing.
                        b.iter(|| black_box($method(black_box(hashes)).to_hex()));
                    },
                );
            };
        }

        bench_method!(iterate_and_update);
        bench_method!(iterate_to_byte_vector);
        bench_method!(copy_to_vector);
        bench_method!(borrow_as_slice);
    }
    group.finish();
}

criterion_group!(benches, bench_hash_list_hash);
criterion_main!(benches);
