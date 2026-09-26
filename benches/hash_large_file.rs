mod utils;

use std::{
    fs::File,
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
use memmap2::Mmap;
use rayon::prelude::*;
use utils::TempDir;

fn hash_file<const PARALLEL: bool>(bytes: &[u8]) -> Hash {
    let mut hasher = Hasher::new();
    // Preserve paq's path/type prefix and its partially filled initial chunk.
    hasher.update(b"directory/file");
    hasher.update(&[0, 0x01]);
    if PARALLEL {
        hasher.update_rayon(bytes);
    } else {
        hasher.update(bytes);
    }
    hasher.finalize()
}

fn hash_files<const PARALLEL: bool>(files: &[Mmap]) -> Vec<Hash> {
    if files.len() == 1 {
        vec![hash_file::<PARALLEL>(&files[0])]
    } else {
        // Simulate multiple active file-hashing tasks in the same Rayon pool.
        files
            .par_iter()
            .map(|file| hash_file::<PARALLEL>(file))
            .collect()
    }
}

fn bench_hash_large_file(c: &mut Criterion) {
    const MIB: usize = 1024 * 1024;
    let mut group = c.benchmark_group("blake3_large_file");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    // Compare both methods around the candidate threshold; no cutoff is imposed.
    let cases = [
        (1, MIB),
        (1, 8 * MIB),
        (1, 16 * MIB - 1),
        (1, 16 * MIB),
        (1, 16 * MIB + 1),
        (1, 64 * MIB),
        (8, 16 * MIB),
    ];
    for (count, size) in cases {
        let name = format!("files-{count}_bytes-{size}");
        let dir = TempDir::new(&format!("bench_large_{name}_{}", std::process::id())).unwrap();
        let files: Vec<_> = (0..count)
            .map(|i| {
                let filename = format!("file-{i}");
                dir.new_file_with_random_data(&filename, size as u64)
                    .unwrap();
                let file = File::open(dir.path().join(filename)).unwrap();
                assert_eq!(file.metadata().unwrap().len(), size as u64);
                // Fixtures remain immutable and mapped until both variants finish.
                unsafe { Mmap::map(&file).unwrap() }
            })
            .collect();

        // Validate hashes and warm mapped pages/the pool outside timing.
        assert_eq!(hash_files::<false>(&files), hash_files::<true>(&files));
        group.throughput(Throughput::Bytes((count * size) as u64));
        group.bench_with_input(BenchmarkId::new("sequential", &name), &files, |b, files| {
            b.iter(|| black_box(hash_files::<false>(black_box(files))));
        });
        group.bench_with_input(BenchmarkId::new("rayon", &name), &files, |b, files| {
            b.iter(|| black_box(hash_files::<true>(black_box(files))));
        });
        // Both methods exclude file creation, opening and mapping. This measures
        // warm mapped-byte hashing, not cold I/O or Windows mmap eligibility.
    }
    group.finish();
}

criterion_group!(benches, bench_hash_large_file);
criterion_main!(benches);
