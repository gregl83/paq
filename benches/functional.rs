#![feature(test)]

#[path="../src/lib.rs"]
mod paq;
mod utils;

extern crate test;
use test::Bencher;

use utils::TempDir;


mod hash_algorithms {
    use super::*;

    #[bench]
    fn bench_blake3_final_hash_iterate_updates(b: &mut Bencher) {
        let file_hashes: Vec<[u8; 32]> = (0..1000)
            .map(|i| {
                blake3::hash(
                    format!("test_file_{i}").as_bytes()
                ).as_bytes().to_owned()
            })
            .collect();

        b.iter(move || {
            let mut hasher = blake3::Hasher::new();

            for file_hash in &file_hashes {
                hasher.update(file_hash);
            }

            hasher.finalize().to_hex()
        });
    }

    #[bench]
    fn bench_blake3_final_hash_array_flatten(b: &mut Bencher) {
        let file_hashes: Vec<[u8; 32]> = (0..1000)
            .map(|i| {
                blake3::hash(
                    format!("test_file_{i}").as_bytes()
                ).as_bytes().to_owned()
            })
            .collect();

        b.iter(move || {
            let flat_bytes: Vec<u8> = file_hashes
                .iter()
                .flat_map(|arr| arr.iter())
                .copied()
                .collect();
            blake3::hash(&flat_bytes).to_hex()
        });
    }

    #[bench]
    fn bench_blake3_final_hash_flat_bytes(b: &mut Bencher) {
        let file_hashes: Vec<[u8; 32]> = (0..1000)
            .map(|i| {
                blake3::hash(
                    format!("test_file_{i}").as_bytes()
                ).as_bytes().to_owned()
            })
            .collect();

        b.iter(move || {
            let mut flat_bytes = Vec::with_capacity(file_hashes.len() * 32);
            for file_hash in &file_hashes {
                flat_bytes.extend_from_slice(file_hash);
            }
            blake3::hash(&flat_bytes).to_hex()
        });
    }

    #[bench]
    fn bench_blake3_final_hash_unsafe_slice(b: &mut Bencher) {
        let file_hashes: Vec<[u8; 32]> = (0..1000)
            .map(|i| {
                blake3::hash(
                    format!("test_file_{i}").as_bytes()
                ).as_bytes().to_owned()
            })
            .collect();

        b.iter(move || {
            if file_hashes.is_empty() {
                return blake3::hash(&[]).to_hex();
            }

            let byte_len = match file_hashes.len().checked_mul(32) {
                Some(len) => len,
                None => {
                    // Fallback for huge vectors
                    let mut flat_bytes = Vec::with_capacity(file_hashes.len() * 32);
                    for file_hash in &file_hashes {
                        flat_bytes.extend_from_slice(file_hash);
                    }
                    return blake3::hash(&flat_bytes).to_hex();
                }
            };

            unsafe {
                let ptr = file_hashes.as_ptr() as *const u8;
                let slice = std::slice::from_raw_parts(ptr, byte_len);
                blake3::hash(slice).to_hex()
            }
        });
    }
}

mod file_size {
    use super::*;

    use std::{
        fs,
        io::Read,
    };

    use memmap2::Mmap;
    use blake3::Hasher;

    const SMALL_FILE_SIZE: u64 = 32_768;
    const MEDIUM_FILE_SIZE: u64 = 163_840;
    const LARGE_FILE_SIZE: u64 = 1_048_576;

    #[bench]
    fn bench_small_file_unbuffered(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_small_file_unbuffered"
        ).unwrap();

        let file_name = "small_file";

        dir.new_file_with_random_data(file_name, SMALL_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::read(&file_path).unwrap();
            hasher.update(&file);
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_small_file_mmap(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_small_file_mmap"
        ).unwrap();

        let file_name = "small_file";

        dir.new_file_with_random_data(file_name, SMALL_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::File::open(&file_path).unwrap();
            let mmap = unsafe { Mmap::map(&file) }.unwrap();
            hasher.update(&mmap);
            *hasher.finalize().as_bytes()
        });
    }

    #[cfg(not(target_os = "windows"))]
    #[bench]
    fn bench_small_file_mmap_advise_sequential(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_small_file_mmap"
        ).unwrap();

        let file_name = "small_file";

        dir.new_file_with_random_data(file_name, SMALL_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::File::open(&file_path).unwrap();
            let mmap = unsafe { Mmap::map(&file) }.unwrap();
            _ = mmap.advise(memmap2::Advice::Sequential);
            hasher.update(&mmap);
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_small_file_buffer(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_small_file_buffer"
        ).unwrap();

        let file_name = "small_file";

        dir.new_file_with_random_data(file_name, SMALL_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let mut file = fs::File::open(&file_path).unwrap();
            let mut buffer = [0; paq::FILE_BUFFER_SIZE];
            loop {
                let buffer_size = file.read(&mut buffer[..]).unwrap();
                if buffer_size == 0 { break; }
                hasher.update(&buffer[..buffer_size]);
            }
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_medium_file_unbuffered(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_medium_file_unbuffered"
        ).unwrap();

        let file_name = "medium_file";

        dir.new_file_with_random_data(file_name, MEDIUM_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::read(&file_path).unwrap();
            hasher.update(&file);
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_medium_file_mmap(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_medium_file_mmap"
        ).unwrap();

        let file_name = "medium_file";

        dir.new_file_with_random_data(file_name, MEDIUM_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::File::open(&file_path).unwrap();
            let mmap = unsafe { Mmap::map(&file) }.unwrap();
            hasher.update(&mmap);
            *hasher.finalize().as_bytes()
        });
    }

    #[cfg(not(target_os = "windows"))]
    #[bench]
    fn bench_medium_file_mmap_advise_sequential(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_medium_file_mmap"
        ).unwrap();

        let file_name = "medium_file";

        dir.new_file_with_random_data(file_name, MEDIUM_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::File::open(&file_path).unwrap();
            let mmap = unsafe { Mmap::map(&file) }.unwrap();
            _ = mmap.advise(memmap2::Advice::Sequential);
            hasher.update(&mmap);
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_medium_file_buffer(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_medium_file_buffer"
        ).unwrap();

        let file_name = "medium_file";

        dir.new_file_with_random_data(file_name, MEDIUM_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let mut file = fs::File::open(&file_path).unwrap();
            let mut buffer = [0; paq::FILE_BUFFER_SIZE];
            loop {
                let buffer_size = file.read(&mut buffer[..]).unwrap();
                if buffer_size == 0 { break; }
                hasher.update(&buffer[..buffer_size]);
            }
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_large_file_unbuffered(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_large_file_unbuffered"
        ).unwrap();

        let file_name = "large_file";

        dir.new_file_with_random_data(file_name, LARGE_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::read(&file_path).unwrap();
            hasher.update(&file);
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_large_file_mmap(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_large_file_mmap"
        ).unwrap();

        let file_name = "large_file";

        dir.new_file_with_random_data(file_name, LARGE_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::File::open(&file_path).unwrap();
            let mmap = unsafe { Mmap::map(&file) }.unwrap();
            hasher.update(&mmap);
            *hasher.finalize().as_bytes()
        });
    }

    #[cfg(not(target_os = "windows"))]
    #[bench]
    fn bench_large_file_mmap_advise_sequential(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_large_file_mmap"
        ).unwrap();

        let file_name = "large_file";

        dir.new_file_with_random_data(file_name, LARGE_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let file = fs::File::open(&file_path).unwrap();
            let mmap = unsafe { Mmap::map(&file) }.unwrap();
            _ = mmap.advise(memmap2::Advice::Sequential);
            hasher.update(&mmap);
            *hasher.finalize().as_bytes()
        });
    }

    #[bench]
    fn bench_large_file_buffer(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_large_file_buffer"
        ).unwrap();

        let file_name = "large_file";

        dir.new_file_with_random_data(file_name, LARGE_FILE_SIZE).unwrap();

        let source = dir.path().canonicalize().unwrap();
        let file_path= source.join(file_name);

        b.iter(|| {
            let mut hasher = Hasher::new();
            let mut file = fs::File::open(&file_path).unwrap();
            let mut buffer = [0; paq::FILE_BUFFER_SIZE];
            loop {
                let buffer_size = file.read(&mut buffer[..]).unwrap();
                if buffer_size == 0 { break; }
                hasher.update(&buffer[..buffer_size]);
            }
            *hasher.finalize().as_bytes()
        });
    }
}

mod lib {
    use super::*;

    #[bench]
    fn bench_hashes_directory_files(b: &mut Bencher) {
        let dir = TempDir::new(
            "bench_hashes_directory_files"
        ).unwrap();

        for i in 0..100 {
            dir.new_file(
                format!("{i}").as_str(),
                format!("{i}-body").as_bytes()
            ).unwrap()
        }

        let source = dir.path().canonicalize().unwrap();

        b.iter(|| paq::hash_source(&source, false));
    }
}
