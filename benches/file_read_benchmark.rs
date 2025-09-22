use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn benchmark_file_read_speed(c: &mut Criterion) {
    let benchmark_file = "assets/benchmark.osu";

    c.bench_function("file_read_parser_process_loop", |b| {
        b.iter(|| {
            let mut parser = ssrrr::file_parser::builder::Parser::new(black_box(benchmark_file));
            parser.process().unwrap();
            let (_cc, _cols, starts, _ends, _types, _od) = parser.get_parsed_data();
            black_box(starts.len())
        })
    });
}

criterion_group!(benches, benchmark_file_read_speed);
criterion_main!(benches);


