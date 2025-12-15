use std::{
    hint::black_box,
    time::Duration,
};

use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use rayon::prelude::ParallelSliceMut;


fn bench_hash_list_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "blake3_final_hash"
    );
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    // rayon warmup (ensure thread pool init)
    let mut warmup: Vec<i32> = (0..64).collect();
    warmup.par_sort_unstable();

    // generate hash list
    let file_hashes: Vec<[u8; 32]> = (0..1000)
        .map(|i| {
            blake3::hash(
                format!("test_file_{i}").as_bytes()
            ).as_bytes().to_owned()
        })
        .collect();

    group.bench_with_input(
        "iterate_and_update",
        &file_hashes,
        |b, file_hashes| {
            let v = file_hashes.clone();
            b.iter(|| {
                let mut hasher = blake3::Hasher::new();

                for file_hash in &v {
                    hasher.update(file_hash);
                }

                black_box(hasher.finalize().to_hex());
            });
        }
    );

    group.bench_with_input(
        "iterate_to_byte_vector",
        &file_hashes,
        |b, file_hashes| {
            let v = file_hashes.clone();
            b.iter(|| {
                let flat_bytes: Vec<u8> = v
                    .iter()
                    .flat_map(|arr| arr.iter())
                    .copied()
                    .collect();

                black_box(blake3::hash(&flat_bytes).to_hex());
            });
        }
    );

    group.bench_with_input(
        "iterate_to_byte_array",
        &file_hashes,
        |b, file_hashes| {
            let v = file_hashes.clone();
            b.iter(|| {
                let mut flat_bytes = Vec::with_capacity(v.len() * 32);

                for file_hash in &v {
                    flat_bytes.extend_from_slice(file_hash);
                }

                black_box(blake3::hash(&flat_bytes).to_hex());
            });
        }
    );

    group.bench_with_input(
        "unsafe_to_slice",
        &file_hashes,
        |b, file_hashes| {
            let v = file_hashes.clone();
            b.iter(|| {
                let byte_len = v.len().checked_mul(32).unwrap();
                unsafe {
                    let ptr = v.as_ptr() as *const u8;
                    let slice = std::slice::from_raw_parts(ptr, byte_len);

                    black_box(blake3::hash(slice).to_hex());
                }
            });
        }
    );

    group.finish();
}

criterion_group!(benches, bench_hash_list_hash);
criterion_main!(benches);
