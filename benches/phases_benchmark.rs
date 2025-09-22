use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn benchmark_phases(c: &mut Criterion) {
    let benchmark_file = "assets/benchmark.osu";

    // Preprocess once
    let map_data = ssrrr::algorithm::process::preprocess::preprocess_file(benchmark_file, "None").expect("preprocess ok");

    // Phase 1
    c.bench_function("phase1_data_prep", |b| {
        b.iter(|| {
            let out = ssrrr::algorithm::process::process::phase1(black_box(&map_data));
            black_box(out.0.len() + out.1.len() + out.2.len())
        })
    });

    let (all_corners, base_corners, a_corners, _key_usage, active_columns, _key_usage_400, anchor) =
        ssrrr::algorithm::process::process::phase1(&map_data);

    // Phase 2
    c.bench_function("phase2_bars", |b| {
        b.iter(|| {
            let out = ssrrr::algorithm::process::process::phase2(
                black_box(&map_data),
                black_box(&active_columns),
                black_box(&a_corners),
                black_box(&base_corners),
                black_box(&all_corners),
                black_box(&anchor),
            );
            black_box(out.0.len() + out.1.len() + out.2.len() + out.3.len() + out.4.len())
        })
    });

    let (jbar, xbar, pbar, abar, rbar, c_arr, ks_arr) =
        ssrrr::algorithm::process::process::phase2(&map_data, &active_columns, &a_corners, &base_corners, &all_corners, &anchor);

    // Phase 3
    c.bench_function("phase3_final_values", |b| {
        b.iter(|| {
            let out = ssrrr::algorithm::process::process::phase3(
                black_box(&jbar),
                black_box(&xbar),
                black_box(&pbar),
                black_box(&abar),
                black_box(&rbar),
                black_box(&c_arr),
                black_box(&ks_arr),
            );
            black_box(out.0.len() + out.1.len() + out.2.len())
        })
    });

    let (_s_all, _t_all, d_all) = ssrrr::algorithm::process::process::phase3(&jbar, &xbar, &pbar, &abar, &rbar, &c_arr, &ks_arr);

    // Phase 4
    c.bench_function("phase4_weighted_aggregation", |b| {
        b.iter(|| {
            let out = ssrrr::algorithm::process::process::phase4(
                black_box(&d_all),
                black_box(&c_arr),
                black_box(&all_corners),
            );
            black_box(out.0 + out.1 + out.2)
        })
    });

    let (p93, p83, wmean) = ssrrr::algorithm::process::process::phase4(&d_all, &c_arr, &all_corners);

    // Phase 5
    c.bench_function("phase5_final_star_rating", |b| {
        b.iter(|| {
            let out = ssrrr::algorithm::process::process::phase5(
                black_box(p93),
                black_box(p83),
                black_box(wmean),
                black_box(&map_data.notes),
                black_box(&map_data.long_notes),
            );
            black_box(out)
        })
    });
}

criterion_group!(benches, benchmark_phases);
criterion_main!(benches);


