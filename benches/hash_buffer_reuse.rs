mod utils;

use std::{fs, hint::black_box, io::Read, path::Path, path::PathBuf, time::Duration};

use blake3::{Hash, Hasher};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memmap2::Mmap;
use paq::{
    FILE_BUFFER_SIZE, MAX_FILE_SIZE_FOR_UNBUFFERED_READ, MIN_FILE_SIZE_FOR_MMAP_READ,
    PATH_BATCH_SIZE,
};
use rayon::prelude::*;
use utils::TempDir;

fn read_buffered(hasher: &mut Hasher, path: &Path, buffer: &mut [u8]) {
    let mut file = fs::File::open(path).unwrap();
    loop {
        let n = file.read(buffer).unwrap();
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
}

fn hash_file<const REUSE: bool>(path: &Path, scratch: &mut [u8]) -> Hash {
    let mut hasher = Hasher::new();
    let size = fs::metadata(path).unwrap().len();
    if size == 0 {
        return hasher.finalize();
    }
    if size < MAX_FILE_SIZE_FOR_UNBUFFERED_READ {
        hasher.update(&fs::read(path).unwrap());
    } else {
        if size > MIN_FILE_SIZE_FOR_MMAP_READ {
            let file = fs::File::open(path).unwrap();
            // Fixtures are immutable until both benchmarks have finished.
            if let Ok(mapped) = unsafe { Mmap::map(&file) } {
                hasher.update(&mapped);
                return hasher.finalize();
            }
        }
        if REUSE {
            read_buffered(&mut hasher, path, scratch);
        } else {
            read_buffered(&mut hasher, path, &mut [0; FILE_BUFFER_SIZE]);
        }
    }
    hasher.finalize()
}

// Prepared entries isolate payload reads and buffer lifetime from traversal,
// path encoding and finalization. None represents the root directory's batch slot.
// Both variants process batches in parallel and entries within each batch serially.
fn hash_batches<const REUSE: bool>(entries: &[Option<PathBuf>]) -> Vec<Hash> {
    entries
        .par_chunks(PATH_BATCH_SIZE)
        .flat_map_iter(|batch| {
            if REUSE {
                let mut buffer = [0; FILE_BUFFER_SIZE];
                batch
                    .iter()
                    .map(move |entry| match entry {
                        Some(path) => hash_file::<true>(path, &mut buffer),
                        None => blake3::hash(b""),
                    })
                    .collect::<Vec<_>>()
            } else {
                batch
                    .iter()
                    .map(|entry| match entry {
                        Some(path) => hash_file::<false>(path, &mut []),
                        None => blake3::hash(b""),
                    })
                    .collect::<Vec<_>>()
            }
        })
        .collect()
}

fn bench_hash_buffer_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_buffer_reuse");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    let mut cases = Vec::new();
    for count in [1, 10, 98, 99, 100, 101, 199, 200] {
        cases.push((format!("files-{count}_4KiB"), vec![4 * 1024; count]));
    }
    for kib in [2, 32, 64, 512] {
        cases.push((format!("files-99_{kib}KiB"), vec![kib * 1024; 99]));
    }
    cases.push(("tiny-only".into(), vec![512; 99]));
    cases.push(("empty-only".into(), vec![0; 99]));
    // Windows retains a 1 GiB mmap threshold; avoid a huge control fixture there.
    #[cfg(not(target_os = "windows"))]
    cases.push(("mmap-only".into(), vec![2 * 1024 * 1024; 99]));
    cases.push((
        "mixed".into(),
        (0..99)
            .map(|i| [64 * 1024, 2 * 1024, 512, 0][i % 4])
            .collect(),
    ));

    for (name, sizes) in cases {
        let dir = TempDir::new(&format!("bench_buffer_{name}_{}", std::process::id())).unwrap();
        let mut entries = vec![None];
        let mut expected = vec![blake3::hash(b"")];
        for (i, size) in sizes.iter().copied().enumerate() {
            let contents: Vec<_> = (0..size).map(|j| ((j + i) % 251) as u8).collect();
            let filename = format!("file-{i}");
            dir.new_file(&filename, &contents).unwrap();
            entries.push(Some(dir.path().join(filename)));
            expected.push(blake3::hash(&contents));
        }
        // Validate full reads and stale-tail handling outside timing; also warm caches/pool.
        assert_eq!(hash_batches::<false>(&entries), expected);
        assert_eq!(hash_batches::<true>(&entries), expected);
        group.throughput(Throughput::Elements(sizes.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("per_file", &name),
            &entries,
            |b, entries| {
                b.iter(|| black_box(hash_batches::<false>(black_box(entries))));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("per_batch", &name),
            &entries,
            |b, entries| {
                b.iter(|| black_box(hash_batches::<true>(black_box(entries))));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_hash_buffer_reuse);
criterion_main!(benches);
