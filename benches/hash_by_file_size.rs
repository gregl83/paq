#[path="../src/lib.rs"]
mod paq;
mod utils;

use std::{
    fs,
    hint::black_box,
    io::Read,
    time::Duration,
};

use blake3::Hasher;
use criterion::{
    BenchmarkId,
    Criterion,
    criterion_group,
    criterion_main,
};
use memmap2::Mmap;
use rayon::prelude::ParallelSliceMut;

use utils::TempDir;


fn bench_hash_by_file_size(c: &mut Criterion) {
    const SMALL_FILE_SIZE: u64 = 32_768;
    const MEDIUM_FILE_SIZE: u64 = 163_840;
    const LARGE_FILE_SIZE: u64 = 1_048_576;

    let mut group = c.benchmark_group(
        "blake3_hash_by_file_size",
    );
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    // rayon warmup (ensure thread pool init)
    let mut warmup: Vec<i32> = (0..64).collect();
    warmup.par_sort_unstable();

    // run benchmarks with various file sizes
    for &n in &[SMALL_FILE_SIZE, MEDIUM_FILE_SIZE, LARGE_FILE_SIZE] {
        // generate file with `n` size
        let dir = TempDir::new(
            format!("bench_file_{n}").as_str()
        ).unwrap();

        let file_name = format!("file_{n}");

        dir.new_file_with_random_data(&file_name, SMALL_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        group.bench_with_input(
            BenchmarkId::new("unbuffered", n),
            &file_path,
            |b, file_path| {
                let p = file_path.clone();
                b.iter(|| {
                    let mut hasher = Hasher::new();
                    let file = fs::read(&p).unwrap();
                    hasher.update(&file);

                    black_box(*hasher.finalize().as_bytes());
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("buffered", n),
            &file_path,
            |b, file_path| {
                let p = file_path.clone();
                b.iter(|| {
                    let mut hasher = Hasher::new();
                    let mut file = fs::File::open(&p).unwrap();
                    let mut buffer = [0; paq::FILE_BUFFER_SIZE];
                    loop {
                        let buffer_size = file.read(&mut buffer[..]).unwrap();
                        if buffer_size == 0 { break; }
                        hasher.update(&buffer[..buffer_size]);
                    }
                    black_box(*hasher.finalize().as_bytes());
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("mmap", n),
            &file_path,
            |b, file_path| {
                let p = file_path.clone();
                b.iter(|| {
                    let mut hasher = Hasher::new();
                    let file = fs::File::open(&p).unwrap();
                    let mmap = unsafe { Mmap::map(&file) }.unwrap();
                    hasher.update(&mmap);

                    black_box(*hasher.finalize().as_bytes());
                })
            }
        );

        #[cfg(not(target_os = "windows"))]
        group.bench_with_input(
            BenchmarkId::new("mmap_advise_sequential", n),
            &file_path,
            |b, file_path| {
                let p = file_path.clone();
                b.iter(|| {
                    let mut hasher = Hasher::new();
                    let file = fs::File::open(&file_path).unwrap();
                    let mmap = unsafe { Mmap::map(&file) }.unwrap();
                    _ = mmap.advise(memmap2::Advice::Sequential);
                    hasher.update(&mmap);

                    black_box(*hasher.finalize().as_bytes());
                })
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_hash_by_file_size);
criterion_main!(benches);
