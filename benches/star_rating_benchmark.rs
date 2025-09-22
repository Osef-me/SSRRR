use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use ssrrr::algorithm::process::preprocess::preprocess_file;
use ssrrr::algorithm::process::process::calculate;

/// Simple benchmark for star rating calculation speed
fn benchmark_star_rating_speed(c: &mut Criterion) {
    let benchmark_file = "assets/benchmark.osu";
    
    // Preprocess the file once to get map data
    let map_data = match preprocess_file(benchmark_file, "None") {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error preprocessing benchmark file: {}", e);
            return;
        }
    };

    println!("Benchmarking with map: {} notes, {} long notes", 
             map_data.note_count(), map_data.long_note_count());

    // Benchmark the calculation function in a loop
    c.bench_function("star_rating_calculation_loop", |b| {
        b.iter(|| {
            let result = calculate(black_box(&map_data));
            black_box(result)
        })
    });
}

criterion_group!(benches, benchmark_star_rating_speed);
criterion_main!(benches);
