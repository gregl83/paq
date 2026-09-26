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
};

// Const parameters keep encoding selection outside the measured hashing work.
fn hash_entry<const TYPED: bool, const SEPARATED: bool, const COMBINED: bool>(
    entry_type: u8,
    path: &[u8],
    payload: &[u8],
) -> Hash {
    let mut hasher = Hasher::new();
    if TYPED && !COMBINED {
        hasher.update(&[entry_type]);
    }
    hasher.update(path);
    if SEPARATED {
        if COMBINED {
            hasher.update(&[0, entry_type]);
        } else {
            hasher.update(&[0]);
        }
    }
    if !payload.is_empty() {
        hasher.update(payload);
    }
    hasher.finalize()
}

fn bench_hash_entry_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_entry_encoding");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));

    // Entry kind, type byte, path bytes, payload bytes. Payloads use one update
    // to isolate encoding overhead from filesystem and read costs.
    let cases = [
        ("directory", 2, 0, 0),
        ("directory", 2, 16, 0),
        ("symlink", 3, 16, 32),
        ("file", 1, 16, 0),
        ("file", 1, 16, 10),
        ("file", 1, 16, 32 * 1024),
        ("file", 1, 16, 1024 * 1024),
        ("file", 1, 128, 10),
        // Path + payload totals 64 or 1,024 bytes; encoding bytes cross the
        // BLAKE3 block or chunk boundary.
        ("file", 1, 16, 48),
        ("file", 1, 16, 1008),
    ];

    for (kind, entry_type, path_len, payload_len) in cases {
        let name = format!("{kind}_path-{path_len}B_payload-{payload_len}B");
        let path: Vec<u8> = b"directory/"
            .iter()
            .copied()
            .cycle()
            .take(path_len)
            .collect();
        let payload: Vec<u8> = if entry_type == 3 {
            b"target/"
                .iter()
                .copied()
                .cycle()
                .take(payload_len)
                .collect()
        } else {
            (0..payload_len).map(|i| (i % 256) as u8).collect()
        };

        macro_rules! bench_encoding {
            ($label:literal, $typed:literal, $separated:literal, $combined:literal) => {{
                // Validate against a single concatenated input outside timing.
                let mut encoded = Vec::new();
                if $typed && !$combined {
                    encoded.push(entry_type);
                }
                encoded.extend_from_slice(&path);
                if $separated {
                    encoded.push(0);
                    if $combined {
                        encoded.push(entry_type);
                    }
                }
                encoded.extend_from_slice(&payload);
                assert_eq!(
                    hash_entry::<$typed, $separated, $combined>(entry_type, &path, &payload),
                    blake3::hash(&encoded),
                    "{} / {}",
                    $label,
                    name
                );

                group.bench_with_input(
                    BenchmarkId::new($label, &name),
                    &(entry_type, path.as_slice(), payload.as_slice()),
                    |b, &(entry_type, path, payload)| {
                        b.iter(|| {
                            black_box(hash_entry::<$typed, $separated, $combined>(
                                black_box(entry_type),
                                black_box(path),
                                black_box(payload),
                            ))
                        });
                    },
                );
            }};
        }

        bench_encoding!("path_and_payload", false, false, false);
        bench_encoding!("path_nul_payload", false, true, false);
        bench_encoding!("type_path_nul_payload", true, true, false);
        bench_encoding!("path_nul_type_payload", true, true, true);
    }

    group.finish();
}

criterion_group!(benches, bench_hash_entry_encoding);
criterion_main!(benches);
