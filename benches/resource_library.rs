//! Resource benchmark binary, not a public paq command.
//! Run with: resource_library SOURCE ITERATIONS [ignore-hidden]
use std::{hint::black_box, path::Path, time::Instant};

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let source = Path::new(&args[1]);
    let iterations: u64 = args[2].to_str().unwrap().parse().unwrap();
    assert!(iterations > 0);
    let ignore_hidden = args.get(3).is_some_and(|arg| arg == "ignore-hidden");
    // Warm the library and its pool (when used) before the measured loop.
    let expected = paq::try_hash_source(source, ignore_hidden).unwrap();
    let start = Instant::now();
    for _ in 0..iterations {
        let actual = black_box(paq::try_hash_source(black_box(source), ignore_hidden).unwrap());
        assert_eq!(actual, expected, "fixture changed during measurement");
    }
    println!("{} {} {:.9}", expected, iterations, start.elapsed().as_secs_f64() * 1000.0);
}
