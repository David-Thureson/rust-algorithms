#![allow(unused_imports)]
#![allow(unused_macros)]

use std::iter;
// use test::black_box;

use criterion::{criterion_group, criterion_main, black_box, Criterion, BenchmarkId, BatchSize, Throughput, PlotConfiguration, AxisScale};

use algorithms::sort::{self, bubble_sort, insertion_sort, merge_sort, quicksort_ptr, quicksort_rayon, quicksort_safe};
use algorithms::sort::test_data::*;
use util::*;

const QUICKSORT_RAYON_MIN_SPLIT_SIZE: u8 = 18;
const QUICKSORT_RAYON_MIN_THREAD_SIZE: u16 = 275;


macro_rules! merge_sort_merge_from_end { ($v:ident) => { merge_sort::merge_sort_merge_from_end($v); } }
macro_rules! merge_sort_merge_in_place { ($v:ident) => { merge_sort::merge_sort_merge_in_place($v); } }
macro_rules! quicksort_rayon { ($v:ident) => { quicksort_rayon::quicksort_rayon($v, QUICKSORT_RAYON_MIN_SPLIT_SIZE, QUICKSORT_RAYON_MIN_THREAD_SIZE); } }
macro_rules! vec_sort_unstable { ($v:ident) => { $v.sort_unstable(); } }

macro_rules! group_bench_with_input {
    ($group:ident, $data_func:ident, $change_var:ident, $v:ident, $($c:ident),*) => {
        $(
            $group.bench_with_input(BenchmarkId::new(stringify!($c), $change_var), &$change_var, |b, &$change_var| {
                b.iter_batched_ref(|| $data_func($change_var), |$v| { $c!($v) }, BatchSize::LargeInput)
            });
        )*
    }
}

pub fn sort_compare_shuffled(c: &mut Criterion) {
    sort_compare_alter_data(c, vec_usize_shuffled, "sort_compare_shuffled");
}

pub fn sort_compare_alter_data(c: &mut Criterion, data_func: fn(usize) -> Vec<usize>, group_name: &str) {
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group(group_name);
    group.plot_config(plot_config);

    for count in vec_powers(15, 20, 2) {
        // for count in vec_powers(6, 1, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group_bench_with_input!(group, data_func, count, v, merge_sort_merge_from_end, merge_sort_merge_in_place, quicksort_rayon, vec_sort_unstable);
    }
    group.finish();
}

criterion_group!(benches,
    sort_compare_shuffled
    );
criterion_main!(benches);

/*
macro_rules! group_bench_with_input {
    ($group:ident, $data_func:ident, $change_var:ident, $v:ident, $c:ident) => {
        $group.bench_with_input(BenchmarkId::new(stringify!($c), $change_var), &$change_var, |b, &$change_var| {
            b.iter_batched_ref(|| $data_func($change_var), |$v| { $c!($v) }, BatchSize::LargeInput)
         })
    }
}

//     ($($element:expr),*) => {
*/