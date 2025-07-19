#![feature(test)]

extern crate test;
use test::Bencher;

mod utils;
use utils::TempDir;

#[bench]
fn bench_hashes_directory_files(b: &mut Bencher) {
    // NOTE - implementation of parallelism increased thread allocation overhead (small dirs increase)

    let alpha_file_name = "alpha";
    let alpha_file_contents = "alpha-body".as_bytes();
    let bravo_file_name = "bravo";
    let bravo_file_contents = "bravo-body".as_bytes();
    let charlie_file_name = "charlie";
    let charlie_file_contents = "charlie-body".as_bytes();
    let one_file_name = "1";
    let one_file_contents = "1-body".as_bytes();
    let nine_file_name = "9";
    let nine_file_contents = "9-body".as_bytes();

    let dir = TempDir::new("bench_hashes_directory_files").unwrap();
    dir.new_file(alpha_file_name, alpha_file_contents).unwrap();
    dir.new_file(bravo_file_name, bravo_file_contents).unwrap();
    dir.new_file(charlie_file_name, charlie_file_contents)
        .unwrap();
    dir.new_file(one_file_name, one_file_contents).unwrap();
    dir.new_file(nine_file_name, nine_file_contents).unwrap();
    let source = dir.path().canonicalize().unwrap();

    b.iter(|| paq::hash_source(&source, false));
}
