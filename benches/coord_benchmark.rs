#![allow(unused_imports)]

use criterion::{criterion_group, criterion_main, black_box, Criterion, BenchmarkId, BatchSize, Throughput, PlotConfiguration, AxisScale};

use algorithms::coord::between_threads::*;

use util::*;
use algorithms::sort::test_data::vec_powers;
use std::sync::atomic::Ordering;

pub fn divide_simple_parallel_find_min_thread_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("divide_simple_parallel_find_min_thread_size");

    let low = 0;
    // let high = 10_000;
    // let high = 100_000;
    let high = 1_000_000;
    let min_split_size = 20;
    let nsec_per_item = 100;
    let inline_fake_work = true;
    let use_atomic_counter = false;
    // for min_thread_size in (80..=120).step_by(2) {
    // for min_thread_size in vec_powers(12, 100, 2) {
    // for min_thread_size in (250..=2000).step_by(250) {
    // for min_thread_size in (200..=4000).step_by(100) {
    for min_thread_size in (2_000..=40_000).step_by(1_000) {
        group.bench_with_input(BenchmarkId::new("divide_simple_parallel", min_thread_size), &min_thread_size, |b, &min_thread_size| {
            b.iter(|| divide_simple_parallel(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter))
        });
    }
    group.finish();
}

pub fn divide_parallel_atomic_counter_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("divide_parallel_atomic_counter_overhead");

    let low = 0;
    let min_split_size = 20;
    let min_thread_size = 1_000;
    let nsec_per_item = 0;
    let inline_fake_work = true;
    let atomic_count_ordering = Some(Ordering::AcqRel);
    // for min_thread_size in (80..=120).step_by(2) {
    for high in vec_powers(15, 100, 2) {
        group.bench_with_input(BenchmarkId::new("divide_parallel_settings_2 (no atomic counter)", high), &high, |b, &high| {
            b.iter(|| divide_parallel_settings_2(low, high as u32, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, None))
        });
        group.bench_with_input(BenchmarkId::new("divide_parallel_settings_2 (with atomic counter)", high), &high, |b, &high| {
            b.iter(|| divide_parallel_settings_2(low, high as u32, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, atomic_count_ordering))
        });
    }
    group.finish();
}

criterion_group!(benches,
    divide_simple_parallel_find_min_thread_size
    // divide_parallel_atomic_counter_overhead
    );
criterion_main!(benches);

// From the main project folder run:
//   cargo +nighly bench
// for all benchmarks or:
//   cargo +nightly bench --bench coord_benchmark
// for just the above group.
