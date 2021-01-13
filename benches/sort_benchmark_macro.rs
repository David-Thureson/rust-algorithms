#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(dead_code)]

use std::iter;
// use test::black_box;
use std::time;

use criterion::{criterion_group, criterion_main, black_box, Criterion, BenchmarkId, BatchSize, Throughput, PlotConfiguration};
use criterion::AxisScale::{Linear, Logarithmic};

use algorithms::sort::{self, bubble_sort, insertion_sort, merge_sort, merge_sort_loop, quicksort_crossbeam, quicksort_ptr, quicksort_rayon, quicksort_safe};
use algorithms::sort::test_data::*;
use util::*;

const MERGE_SORT_LOOP_MIN_SPLIT_SIZE: u8 = 0;
const MERGE_SORT_LOOP_MAX_THREADS: u8 = 2;
const QUICKSORT_CROSSBEAM_MIN_SPLIT_SIZE: u8 = 18;
const QUICKSORT_CROSSBEAM_MIN_THREAD_SIZE: u32 = 275;
const QUICKSORT_RAYON_MIN_SPLIT_SIZE: u8 = 14;
const QUICKSORT_RAYON_MIN_THREAD_SIZE: u16 = 275;

macro_rules! sequence_as {
    ($change_expr:expr, $tt:tt) => {
        // $change_expr.iter().map(|x| *x as $tt)
        $change_expr.iter().map(|x| $tt::try_from(*x).unwrap())
    }
}

macro_rules! merge_sort_merge_from_end      { ($v:ident) => { merge_sort::merge_sort_merge_from_end           ($v); } }
macro_rules! merge_sort_merge_in_place      { ($v:ident) => { merge_sort::merge_sort_merge_in_place           ($v); } }
macro_rules! merge_sort_loop                { ($v:ident) => { merge_sort_loop::merge_sort_loop                ($v, MERGE_SORT_LOOP_MIN_SPLIT_SIZE, MERGE_SORT_LOOP_MAX_THREADS); } }
macro_rules! merge_sort_loop_vec            { ($v:ident) => { merge_sort_loop::merge_sort_loop_vec            ($v, MERGE_SORT_LOOP_MIN_SPLIT_SIZE, MERGE_SORT_LOOP_MAX_THREADS); } }
macro_rules! quicksort_rnd_3_ptr_with_limit { ($v:ident) => { quicksort_ptr::quicksort_rnd_3_ptr_with_limit   ($v, 15); } }
macro_rules! quicksort_parallel_ptr         { ($v:ident) => { quicksort_ptr::quicksort_parallel_ptr           ($v, 0.0, 25_000, false, 0); } }
macro_rules! quicksort_crossbeam            { ($v:ident) => { quicksort_crossbeam::quicksort_crossbeam        ($v, QUICKSORT_CROSSBEAM_MIN_SPLIT_SIZE, QUICKSORT_CROSSBEAM_MIN_THREAD_SIZE); } }
macro_rules! quicksort_crossbeam_minimal    { ($v:ident) => { quicksort_crossbeam::quicksort_crossbeam_minimal($v); } }
macro_rules! quicksort_rayon_minimal        { ($v:ident) => { quicksort_rayon::quicksort_rayon_minimal        ($v); } }
macro_rules! quicksort_rayon                { ($v:ident) => { quicksort_rayon::quicksort_rayon                ($v, QUICKSORT_RAYON_MIN_SPLIT_SIZE, QUICKSORT_RAYON_MIN_THREAD_SIZE); } }
macro_rules! vec_sort_unstable              { ($v:ident) => { $v.sort_unstable(); } }

macro_rules! merge_sort_loop_var     { ($v:ident, $min_split_size:ident, $thread_arg:ident) => { merge_sort_loop::merge_sort_loop        ($v, $min_split_size, $thread_arg); } }
macro_rules! merge_sort_loop_vec_var { ($v:ident, $min_split_size:ident, $thread_arg:ident) => { merge_sort_loop::merge_sort_loop_vec    ($v, $min_split_size, $thread_arg); } }
macro_rules! quicksort_rayon_var     { ($v:ident, $min_split_size:ident, $thread_arg:ident) => { quicksort_rayon::quicksort_rayon        ($v, $min_split_size, $thread_arg); } }
macro_rules! quicksort_crossbeam_var { ($v:ident, $min_split_size:ident, $thread_arg:ident) => { quicksort_crossbeam::quicksort_crossbeam($v, $min_split_size, $thread_arg); } }

macro_rules! sort_compare {
    ($name:ident, $data_func:expr, $change_expr:expr, $scale:expr, $($function_under_test:ident),*) => {
            pub fn $name(c: &mut Criterion) {
                let plot_config = PlotConfiguration::default().summary_scale($scale);

                let mut group = c.benchmark_group(stringify!($name));
                group.plot_config(plot_config);

                for count in $change_expr {
                    group.throughput(Throughput::Elements(count as u64));
                    $(
                        group.bench_with_input(BenchmarkId::new(stringify!($function_under_test), count), &count, |b, &count| {
                            b.iter_batched_ref(|| $data_func(count), |v| { $function_under_test!(v) }, BatchSize::LargeInput)
                        });
                    )*
                }
                group.finish();
            }
    }
}

macro_rules! sort_compare_min_split_size {
    ($name:ident, $data_func:expr, $count:expr, $thread_arg:expr, $change_expr:expr, $scale:expr, $($function_under_test:ident),*) => {
            pub fn $name(c: &mut Criterion) {
                let plot_config = PlotConfiguration::default().summary_scale($scale);

                let mut group = c.benchmark_group(stringify!($name));
                group.plot_config(plot_config);

                let count = $count;
                let thread_arg = $thread_arg;
                for min_split_size in $change_expr {
                    $(
                        group.bench_with_input(BenchmarkId::new(stringify!($function_under_test), min_split_size), &min_split_size, |b, &min_split_size| {
                            b.iter_batched_ref(|| $data_func(count), |v| { $function_under_test!(v, min_split_size, thread_arg) }, BatchSize::LargeInput)
                        });
                    )*
                }
                group.finish();
            }
    }
}

macro_rules! sort_compare_thread_arg {
    ($name:ident, $data_func:expr, $count:expr, $min_split_size:expr, $change_expr:expr, $scale:expr, $($function_under_test:ident),*) => {
            pub fn $name(c: &mut Criterion) {
                let plot_config = PlotConfiguration::default().summary_scale($scale);

                let mut group = c.benchmark_group(stringify!($name));
                group.plot_config(plot_config);

                let count = $count;
                let min_split_size = $min_split_size;
                for thread_arg in $change_expr {
                    $(
                        group.bench_with_input(BenchmarkId::new(stringify!($function_under_test), thread_arg), &thread_arg, |b, &thread_arg| {
                            b.iter_batched_ref(|| $data_func(count), |v| { $function_under_test!(v, min_split_size, thread_arg) }, BatchSize::LargeInput)
                        })
                        //.measurement_time(time::Duration::from_millis(5000))
                        //.sample_size(10)
                        ;
                    )*
                }
                group.finish();
            }
    }
}

sort_compare!{ sort_compare_shuffled, vec_usize_shuffled,
    // vec_powers(15, 20, 2),
    vec_powers(5, 100, 2),
    Logarithmic,
    merge_sort_merge_from_end, merge_sort_merge_in_place,
    merge_sort_loop, merge_sort_loop_vec,
    quicksort_rnd_3_ptr_with_limit, quicksort_parallel_ptr,
    quicksort_crossbeam_minimal, quicksort_rayon_minimal,
    quicksort_crossbeam, quicksort_rayon, vec_sort_unstable }

sort_compare_min_split_size!{ quicksort_rayon_find_min_split_size,     vec_usize_shuffled, 1_000, 12_000, 1..50,              Linear, quicksort_rayon_var }
sort_compare_min_split_size!{ quicksort_crossbeam_find_min_split_size, vec_usize_shuffled, 1_000, 12_000, (5..35).step_by(1), Linear, quicksort_crossbeam_var }
sort_compare_min_split_size!{ merge_sort_loop_find_min_split_size,     vec_usize_shuffled, 1_000, 1, (1..50).step_by(1),      Linear, merge_sort_loop_var }
// sort_compare_min_thread_size!{ quicksort_crossbeam_find_min_thread_size, vec_usize_shuffled, 10_000, QUICKSORT_CROSSBEAM_MIN_SPLIT_SIZE, sequence_as!(vec_powers(10, 1_000, 2), u16), Logarithmic, quicksort_crossbeam_var }
sort_compare_thread_arg!{ merge_sort_loop_find_max_threads,            vec_usize_shuffled,   1_000, MERGE_SORT_LOOP_MIN_SPLIT_SIZE,     vec_powers(7, 1, 2),             Linear, merge_sort_loop_var, merge_sort_loop_vec_var }
sort_compare_thread_arg!{ quicksort_crossbeam_find_min_thread_size,    vec_usize_shuffled, 200_000, QUICKSORT_CROSSBEAM_MIN_SPLIT_SIZE, (40_000..70_000).step_by(1_000), Linear, quicksort_crossbeam_var }

/*
fn quicksort_rayon_find_min_split_size_working(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_rayon_find_min_split_size");
    let count = 100;
    let min_thread_size = 1_000;
    // for limit in vec_powers(7, 5, 2) {
    for min_split_size in 1..=5 {
        group.bench_with_input(BenchmarkId::new("quicksort_rayon", min_split_size), &min_split_size, |b, &min_split_size| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_rayon::quicksort_rayon(v, min_split_size, min_thread_size); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}
*/

criterion_group!(benches,
    sort_compare_shuffled
    // merge_sort_loop_find_min_split_size
    // merge_sort_loop_find_max_threads
    // quicksort_rayon_find_min_split_size
    // quicksort_crossbeam_find_min_split_size
    // quicksort_crossbeam_find_min_thread_size
    );
criterion_main!(benches);

