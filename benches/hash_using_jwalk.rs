mod utils;

use std::{
    fs,
    io::prelude::*,
    path::Path,
    hint::black_box,
    time::Duration,
};

pub use arrayvec::ArrayString;
use blake3::Hasher;
use criterion::{
    Criterion,
    criterion_group,
    criterion_main,
};
use jwalk::{
    DirEntry,
    WalkDir,
};
use memmap2::Mmap;
use rayon::prelude::*;

use utils::TempDir;

/*
IMPORTANT:

Benchmark uses refactor of lib mirror from v1.4.0 for the sake of reproducibility.
*/


pub const PATH_BATCH_SIZE: usize = 100;
pub const MAX_FILE_SIZE_FOR_UNBUFFERED_READ: u64 = 1024 + 1;
#[cfg(not(target_os = "windows"))]
pub const MIN_FILE_SIZE_FOR_MMAP_READ: u64 = 1024 * 1024 - 1;
#[cfg(target_os = "windows")]
pub const MIN_FILE_SIZE_FOR_MMAP_READ: u64 = 1024 * 1024 * 1024 - 1;
#[cfg(not(target_os = "windows"))]
pub const FILE_BUFFER_SIZE: usize = 32 * 1024;
#[cfg(target_os = "windows")]
pub const FILE_BUFFER_SIZE: usize = 128 * 1024;


fn buffer_file_to_hasher(hasher: &mut Hasher, path: &Path) {
    let mut file = fs::File::open(path).unwrap();
    let mut buffer = [0; FILE_BUFFER_SIZE];
    loop {
        let buffer_size = file.read(&mut buffer[..]).unwrap();
        if buffer_size == 0 { break; }
        hasher.update(&buffer[..buffer_size]);
    }
}

fn hash_path(root: &Path, entry: &DirEntry<((), ())>) -> [u8; 32] {
    let path = entry.path();
    let source_path = path.strip_prefix(root).unwrap().to_str().unwrap();
    let source_type = entry.file_type();

    let mut hasher = Hasher::new();
    // hash paths for fs changes other than file content (must be relative to root)
    #[cfg(target_family = "unix")]
    {
        hasher.update(source_path.as_bytes());
    }
    #[cfg(target_family = "windows")]
    {
        hasher.update(source_path.replace("\\", "/").as_bytes());
    }

    if source_type.is_symlink() {
        // for symlinks add hash of target path
        let symlink_target = fs::read_link(&path).unwrap();
        #[cfg(target_family = "unix")]
        {
            hasher.update(symlink_target.to_str().unwrap().as_bytes());
        }
        #[cfg(target_family = "windows")]
        {
            hasher.update(
                symlink_target
                    .to_str()
                    .unwrap()
                    .replace("\\", "/")
                    .as_bytes(),
            );
        }
    } else if source_type.is_file() {
        // for files, add contents to hasher
        let metadata = entry.metadata().unwrap();
        let file_size = metadata.len();
        if file_size == 0 {
            // empty file, return immediately
            return *hasher.finalize().as_bytes();
        } else if file_size < MAX_FILE_SIZE_FOR_UNBUFFERED_READ {
            // small file read using unbuffered
            let file = fs::read(path).unwrap();
            hasher.update(&file);
        } else if file_size > MIN_FILE_SIZE_FOR_MMAP_READ {
            // large size files read using mmap or fail to buffered read
            let file = fs::File::open(&path).unwrap();
            match unsafe { Mmap::map(&file) } {
                Ok(mmap) => { hasher.update(&mmap); },
                Err(_) => { buffer_file_to_hasher(&mut hasher, &path) ; },
            }
        } else {
            // medium file size read using buffer
            buffer_file_to_hasher(&mut hasher, &path);
        }
    }

    *hasher.finalize().as_bytes()
}

fn get_hashes_root(file_hashes: Vec<[u8; 32]>) -> ArrayString<64> {
    let mut flattened_bytes = Vec::with_capacity(file_hashes.len() * 32);

    for file_hash in &file_hashes {
        flattened_bytes.extend_from_slice(file_hash);
    }

    blake3::hash(&flattened_bytes).to_hex()
}

/// Hash file system source.
///
/// Source **must** be a path to a file or directory.
///
/// Uses `blake3` hashing algorithm.
///
/// ```
/// use paq;
///
/// let source = std::path::PathBuf::from("example");
/// let ignore_hidden = true;
/// let source_hash: paq::ArrayString<64> = paq::hash_source(&source, ignore_hidden);
///
/// assert_eq!(&source_hash[..], "d7d25c9b2fdb7391e650085a985ad0d892c7f0dd5edd32c7ccdb4b0d1c34c430");
/// ```
pub fn hash_source(source: &Path, ignore_hidden: bool) -> ArrayString<64> {
    // construct parallel file system walker (unordered)
    let walker = WalkDir::new(source)
        .skip_hidden(ignore_hidden)
        .follow_links(false);

    // construct iterator that retrieves system path batches using walker
    let mut walker_iter = walker.into_iter();
    let batch_iter = std::iter::from_fn(move || {
        let mut batch = Vec::with_capacity(PATH_BATCH_SIZE);
        for _ in 0..PATH_BATCH_SIZE {
            match walker_iter.next() {
                Some(Ok(entry)) => batch.push(entry),
                Some(Err(e)) => panic!("Critical: Failed to traverse directory: {e}"),
                None => break,
            }
        }
        if batch.is_empty() { None } else { Some(batch) }
    });

    // run hashing pipeline using parallel batching
    let mut hashes: Vec<[u8; 32]> = batch_iter
        .par_bridge()
        .flat_map_iter(|batch| {
            // process the whole batch (low lock contention)
            batch.into_iter().map(|entry| hash_path(source, &entry))
        })
        .collect();

    // parallel sort using default rayon MAX_SEQUENTIAL threshold (2k items)
    hashes.par_sort_unstable();

    get_hashes_root(hashes)
}


fn bench_paq_jwalk_library(c: &mut Criterion) {
    let mut group = c.benchmark_group(
        "hash_source_using_jwalk",
    );
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    let dir = TempDir::new(
        "bench_hashes_directory_files"
    ).unwrap();

    for i in 0..1000 {
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
            b.iter(|| hash_source(
                black_box(source),
                false
            ))
        },
    );

    group.finish();
}

criterion_group!(benches, bench_paq_jwalk_library);
criterion_main!(benches);
