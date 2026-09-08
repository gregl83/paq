mod utils;

use std::{fs, hint::black_box, io::Read, path::Path, time::Duration};

use blake3::Hasher;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memmap2::Mmap;
use utils::TempDir;

fn unbuffered(path: &Path) -> blake3::Hash {
    blake3::hash(&fs::read(path).unwrap())
}

fn buffered(path: &Path) -> blake3::Hash {
    let mut hasher = Hasher::new();
    let mut file = fs::File::open(path).unwrap();
    let mut buffer = [0; paq::FILE_BUFFER_SIZE];
    loop {
        let n = file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    hasher.finalize()
}

fn mapped(path: &Path) -> blake3::Hash {
    let file = fs::File::open(path).unwrap();
    // The fixture is immutable for the entire measurement.
    let mmap = unsafe { Mmap::map(&file) }.unwrap();
    blake3::hash(&mmap)
}

#[cfg(not(target_os = "windows"))]
fn advised(path: &Path) -> blake3::Hash {
    let file = fs::File::open(path).unwrap();
    let mmap = unsafe { Mmap::map(&file) }.unwrap();
    let _ = mmap.advise(memmap2::Advice::Sequential);
    blake3::hash(&mmap)
}

fn bench_hash_by_file_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_hash_by_file_size");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    black_box(rayon::current_num_threads());
    let buffer = paq::FILE_BUFFER_SIZE as u64;
    let mut sizes = vec![
        0,
        1,
        63,
        64,
        65,
        1_023,
        1_024,
        1_025,
        buffer - 1,
        buffer,
        buffer + 1,
        163_840,
        1_048_575,
        1_048_576,
        1_048_577,
    ];
    // Windows production mmap starts at 1 GiB; explicitly opt into its cost.
    if std::env::var_os("PAQ_BENCH_LARGE").is_some() {
        sizes.extend([64 * 1_048_576, 256 * 1_048_576]);
        #[cfg(target_os = "windows")]
        sizes.extend([1_073_741_823, 1_073_741_824, 1_073_741_825]);
    }
    sizes.sort_unstable();
    sizes.dedup();
    for n in sizes {
        let dir = TempDir::new(&format!("bench_file_{n}")).unwrap();
        dir.new_file_with_random_data("file", n).unwrap();
        let path = dir.path().join("file");
        assert_eq!(fs::metadata(&path).unwrap().len(), n);
        let expected = buffered(&path);
        let outer = blake3::hash(expected.as_bytes()).to_hex();
        assert_eq!(paq::try_hash_source(&path, false).unwrap(), outer);
        group.throughput(Throughput::Bytes(n));
        let mut methods: Vec<(&str, fn(&Path) -> blake3::Hash)> =
            vec![("unbuffered", unbuffered), ("buffered", buffered)];
        // Empty mappings are unsupported; production's empty shortcut is timed below.
        if n != 0 {
            methods.push(("mmap", mapped));
            #[cfg(not(target_os = "windows"))]
            methods.push(("mmap_advise_sequential", advised));
        }
        for (name, hash) in methods {
            assert_eq!(hash(&path), expected, "{name}/{n}");
            group.bench_with_input(BenchmarkId::new(name, n), &path, |b, path| {
                b.iter(|| black_box(hash(black_box(path))));
            });
        }
        group.bench_with_input(BenchmarkId::new("current_library", n), &path, |b, path| {
            b.iter(|| black_box(paq::try_hash_source(black_box(path), false).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_hash_by_file_size);
criterion_main!(benches);
