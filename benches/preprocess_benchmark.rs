use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn benchmark_preprocess_speed(c: &mut Criterion) {
    let benchmark_file = "assets/benchmark.osu";

    c.bench_function("preprocess_file_loop", |b| {
        b.iter(|| {
            let map = ssrrr::algorithm::process::preprocess::preprocess_file(black_box(benchmark_file), "None").unwrap();
            black_box(map.total_duration)
        })
    });
}

criterion_group!(benches, benchmark_preprocess_speed);
criterion_main!(benches);


