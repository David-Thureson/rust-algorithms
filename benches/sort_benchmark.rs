#![allow(unused_imports)]

use std::iter;
// use test::black_box;

use criterion::{criterion_group, criterion_main, black_box, Criterion, BenchmarkId, BatchSize, Throughput, PlotConfiguration, AxisScale};

use algorithms::sort::{self, bubble_sort, insertion_sort, merge_sort, quicksort_ptr, quicksort_rayon, quicksort_safe};
use algorithms::sort::test_data::*;
use util::*;

const QUICKSORT_RAYON_MIN_SPLIT_SIZE: u8 = 18;
const QUICKSORT_RAYON_MIN_THREAD_SIZE: u16 = 275;

macro_rules! quicksort_rayon { () => { quicksort_rayon::quicksort_rayon(v, QUICKSORT_RAYON_MIN_SPLIT_SIZE, QUICKSORT_RAYON_MIN_THREAD_SIZE); } }
macro_rules! vec_sort_unstable { () => { v.sort_unstable(); } }
macro_rules! group_bench_with_input {
    ($change_var:ident, $c:ident) => {
        group.bench_with_input(BenchmarkId::new(stringify!($c), $change_var), &$change_var, |b, &$change_var| {
            b.iter_batched_ref(|| data_func(count), |v| { $c!() }, BatchSize::LargeInput)
         })
    }
}

pub fn vectors_for_merge_1_000(c: &mut Criterion) {
    vectors_for_merge_benchmark(c, 1_000);
}

pub fn vectors_for_merge_10_000(c: &mut Criterion) {
    vectors_for_merge_benchmark(c, 10_000);
}

pub fn vectors_for_merge_benchmark(c: &mut Criterion, count: usize) {
    c.bench_function(&format!("vectors_for_merge({})", util::format::format_count(count)), |b|
        b.iter(|| vectors_for_merge(black_box(count)))
    );
}

pub fn merge_1_000(c: &mut Criterion) {
    merge_benchmark(c, 1_000);
}

pub fn merge_10_000(c: &mut Criterion) {
    merge_benchmark(c, 10_000);
}

pub fn merge_benchmark(c: &mut Criterion, count: usize) {
    c.bench_function(&format!("merge({})", util::format::format_count(count)), |b|
        b.iter(|| {
            let (mut v1, mut v2) = vectors_for_merge(count);
            merge_sort::merge(&mut v1, &mut v2);
            // Return a value from the closure so the processing is not optimized out by the
            // compiler.
            v1
        })
    );
}

pub fn merge_sort_100(c: &mut Criterion) {
    merge_sort_benchmark(c, 100);
}

pub fn merge_sort_1_000(c: &mut Criterion) {
    merge_sort_benchmark(c, 1_000);
}

pub fn merge_sort_10_000(c: &mut Criterion) {
    merge_sort_benchmark(c, 10_000);
}

pub fn merge_sort_100_000(c: &mut Criterion) {
    merge_sort_benchmark(c, 100_000);
}

pub fn merge_sort_1_000_000(c: &mut Criterion) {
    merge_sort_benchmark(c, 1_000_000);
}

pub fn merge_sort_benchmark(c: &mut Criterion, count: usize) {
    c.bench_function(&format!("merge_sort({})", util::format::format_count(count)), |b|
        b.iter_batched_ref(|| vec_usize_shuffled(count),
                           |v| {
            merge_sort(v);
            // Return a value from the closure so the processing is not optimized out by the
            // compiler.
        },
        BatchSize::LargeInput)
    );
}

pub fn merge_sort_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_sort");
    for count in vec_powers(10, 1, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count),
                               |v| {
                                   merge_sort(v);
                               },
                               BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn merge_sort_vs_rust_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_sort_vs_rust_sort");
    for count in vec_powers(10, 1, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("Vec::sort_unstable", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { v.sort_unstable(); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn sort_compare(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("sort_compare");
    group.plot_config(plot_config);

    for count in vec_powers(10, 1, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_with_bubble", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_with_bubble(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_skip_match", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_skip_match(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_merge_from_end", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_merge_from_end(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_merge_in_place", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_merge_in_place(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("bubble_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { bubble_sort::bubble_sort(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("insertion_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { insertion_sort::insertion_sort(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("Vec::sort_unstable", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { v.sort_unstable(); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn sort_compare_shuffled(c: &mut Criterion) {
    sort_compare_alter_data(c, vec_usize_shuffled, "sort_compare_shuffled");
}

pub fn sort_compare_ordered(c: &mut Criterion) {
    sort_compare_alter_data(c, vec_usize_ordered, "sort_compare_ordered");
}

pub fn sort_compare_reversed(c: &mut Criterion) {
    sort_compare_alter_data(c, vec_usize_reversed, "sort_compare_reversed");
}

pub fn sort_compare_alter_data(c: &mut Criterion, data_func: fn(usize) -> Vec<usize>, group_name: &str) {
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group(group_name);
    group.plot_config(plot_config);

    // for count in vec_powers(4, 125_000, 2) {
    // for count in vec_powers(7, 1, 2) {
    // for count in 1..=20 {
    for count in vec_powers(15, 20, 2) {
    // for count in vec_powers(6, 1, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| data_func(count), |v| { merge_sort::merge_sort_merge_from_end(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_merge_in_place", count), &count, |b, &count| {
            b.iter_batched_ref(|| data_func(count), |v| { merge_sort::merge_sort_merge_in_place(v); }, BatchSize::LargeInput)
        });
        // group.bench_with_input(BenchmarkId::new("bubble_sort", count), &count, |b, &count| {
        //     b.iter_batched_ref(|| data_func(count), |v| { bubble_sort(v); }, BatchSize::LargeInput)
        // });
        // group.bench_with_input(BenchmarkId::new("bubble_sort_ptr", count), &count, |b, &count| {
        //     b.iter_batched_ref(|| data_func(count), |v| { bubble_sort_ptr(&mut v[..]); }, BatchSize::LargeInput)
        // });
        // group.bench_with_input(BenchmarkId::new("insertion_sort", count), &count, |b, &count| {
        //     b.iter_batched_ref(|| data_func(count), |v| { insertion_sort(v); }, BatchSize::LargeInput)
        // });
        group.bench_with_input(BenchmarkId::new("quicksort", count), &count, |b, &count| {
            b.iter_batched_ref(|| data_func(count), |v| { quicksort_ptr::quicksort_rnd_3_ptr_with_limit(v, 15); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("quicksort (parallel ptr)", count), &count, |b, &count| {
            b.iter_batched_ref(|| data_func(count), |v| { quicksort_ptr::quicksort_parallel_ptr(v, 0.0, 25_000, false, 0); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("quicksort_rayon_minimal", count), &count, |b, &count| {
            b.iter_batched_ref(|| data_func(count), |v| { quicksort_rayon::quicksort_rayon_minimal(v); }, BatchSize::LargeInput)
        });
        group_bench_with_input!(count, quicksort_rayon);
        // group.bench_with_input(BenchmarkId::new("quicksort_rayon", count), &count, |b, &count| {
        //     b.iter_batched_ref(|| data_func(count), |v| { quicksort_rayon::quicksort_rayon(v, QUICKSORT_RAYON_MIN_SPLIT_SIZE, QUICKSORT_RAYON_MIN_THREAD_SIZE); }, BatchSize::LargeInput)
        // });
        /*
        group.bench_with_input(BenchmarkId::new("quicksort_rayon", count), &count,
                               |b, &count|
                                   {
                                       b.iter_batched_ref(|| data_func(count),
                                                          |v|
                                                              {
                                                                  quicksort_rayon::quicksort_rayon(v,
                                                                                                   QUICKSORT_RAYON_MIN_SPLIT_SIZE,
                                                                                                   QUICKSORT_RAYON_MIN_THREAD_SIZE)
                                                              },
                                                          BatchSize::LargeInput)
                                   });
        */
        group.bench_with_input(BenchmarkId::new("Vec_sort_unstable", count), &count, |b, &count| {
            b.iter_batched_ref(|| data_func(count), |v| { v.sort_unstable(); }, BatchSize::LargeInput)
        });
        // group.bench_with_input(BenchmarkId::new("merge_in_place", count), &count, |b, &count| {
        //     b.iter_batched_ref(|| data_func(count), |v| { merge_in_place(&mut v[..], 1); }, BatchSize::LargeInput)
        //});
    }
    group.finish();
}

pub fn sort_find_crossover(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_find_crossover");

    for count in 1..30 {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("bubble_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { bubble_sort::bubble_sort(v); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn merge_sort_find_crossover_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_sort_find_crossover_point");

    let count = 1_000;
    for crossover_point in 1..50 {
        group.bench_with_input(BenchmarkId::new("merge_sort_with_bubble", crossover_point), &crossover_point, |b, &crossover_point| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_with_bubble_set_crossover(v, crossover_point); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn quicksort_find_crossover_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_find_crossover_point");

    let count = 1_000;
    for crossover_point in (2..=40).step_by(2) {
        group.bench_with_input(BenchmarkId::new("quicksort", crossover_point), &crossover_point, |b, &crossover_point| {
            // b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_with_crossover(v, crossover_point); }, BatchSize::LargeInput)
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_ptr::quicksort_parallel_ptr(v, 0.0, 25_000, false, crossover_point); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn quicksort_compare_pointer(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_compare_pointer");

    let limit = 20;
    for count in vec_powers(4, 10, 10) {
        group.bench_with_input(BenchmarkId::new("quicksort_rnd_3", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_safe::quicksort_rnd_3_with_limit(v, limit); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("quicksort_rnd_3_ptr", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_ptr::quicksort_rnd_3_ptr_with_limit(v, limit); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn quicksort_find_limit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_find_limit");

    let count = 1_000;
    for limit in vec_powers(7, 5, 2) {
        group.bench_with_input(BenchmarkId::new("quicksort (shuffled)", limit), &limit, |b, &limit| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_ptr::quicksort_rnd_3_ptr_with_limit(v, limit); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("quicksort (ordered)", limit), &limit, |b, &limit| {
            b.iter_batched_ref(|| vec_usize_ordered(count), |v| { quicksort_ptr::quicksort_rnd_3_ptr_with_limit(v, limit); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("quicksort (reversed)", limit), &limit, |b, &limit| {
            b.iter_batched_ref(|| vec_usize_reversed(count), |v| { quicksort_ptr::quicksort_rnd_3_ptr_with_limit(v, limit); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn quicksort_get_parent_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_get_parent_overhead");

    let limit = 15;
    // for count in [10, 21, 100, 201, 1_000, 2_001, 10_000, 20_001].iter() {
    for count in vec_powers(15, 10, 2) {
        group.bench_with_input(BenchmarkId::new("quicksort (shuffled)", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_ptr::quicksort_rnd_3_ptr_with_limit(v, limit); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

// 25,000 is good for 100,000 items.
// 35,000 is good for 1,000,000 items.
pub fn quicksort_find_minimum_thread_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_find_minimum_thread_size");

    // let count = 100_000;
    let count = 1_000_000;
    // for thread_size in vec_powers(7, 1000, 2) {
    for thread_size in (35_000..=55_000).step_by(4000) {
        group.bench_with_input(BenchmarkId::new("quicksort_parallel_ptr", thread_size), &thread_size, |b, &thread_size| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_ptr::quicksort_parallel_ptr(v, 0.0, thread_size, false, 11); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

/*
pub fn quicksort_counter_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_counter_overhead");

    let thread_fraction = 0.0; 
    let thread_size = 25_000;
    for count in vec_powers(5, 10, 10) {
        group.bench_with_input(BenchmarkId::new("quicksort_parallel_ptr (no counter)", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_parallel_ptr(v, thread_fraction, thread_size, false); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("quicksort_parallel_ptr (with counter)", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_parallel_ptr(v, thread_fraction, thread_size, true); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}
*/

pub fn quicksort_rayon_find_min_split_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_rayon_find_min_split_size");
    let count = 100;
    let min_thread_size = 1_000;
    // for limit in vec_powers(7, 5, 2) {
    for min_split_size in 1..=50 {
        group.bench_with_input(BenchmarkId::new("quicksort_rayon", min_split_size), &min_split_size, |b, &min_split_size| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_rayon::quicksort_rayon(v, min_split_size, min_thread_size); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn quicksort_rayon_find_min_thread_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("quicksort_rayon_find_min_thread_size");
    let count = 10_000;
    let min_split_size = 18;
    // for min_thread_size in vec_powers(6, 20, 2) {
    // for min_thread_size in (20..=100).step_by(4) {
    // for min_thread_size in 40..=80 {
    for min_thread_size in (20..=800).step_by(5) {
            group.bench_with_input(BenchmarkId::new("quicksort_rayon", min_thread_size), &min_thread_size, |b, &min_thread_size| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { quicksort_rayon::quicksort_rayon(v, min_split_size, min_thread_size); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn merge_sort_skip_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_sort_skip_merge");

    for count in vec_powers(7, 1, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_with_bubble", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_with_bubble(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_skip_match", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_skip_match(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_merge_from_end", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_merge_from_end(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_test_only_no_merge", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_test_only_no_merge(v); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_sort_merge_in_place", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { merge_sort::merge_sort_merge_in_place(&mut v[..]); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn merge_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_compare");

    for count in vec_powers(11, 2, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge", count), &count, |b, &count| {
            b.iter_batched_ref(|| vectors_for_merge(count), |(v1, v2)| { merge_sort::merge(v1, v2); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_from_end", count), &count, |b, &count| {
            b.iter_batched_ref(|| vectors_for_merge(count), |(v1, v2)| { merge_sort::merge_from_end(v1, v2); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_in_place", count), &count, |b, &count| {
            b.iter_batched_ref(|| vector_for_merge_in_place(count), |(v, mid)| { merge_sort::merge_in_place(&mut v[..], *mid); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_in_place_track_start", count), &count, |b, &count| {
            b.iter_batched_ref(|| vector_for_merge_in_place(count), |(v, mid)| { merge_sort::merge_in_place_track_start(&mut v[..], *mid); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_pointers", count), &count, |b, &count| {
            b.iter_batched_ref(|| vector_for_merge_in_place(count), |(v, mid)| { merge_sort::merge_pointers(&mut v[..], *mid); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn merge_direction(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_direction");

    for count in vec_powers(5, 10, 2) {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("merge_from_end", count), &count, |b, &count| {
            b.iter_batched_ref(|| vectors_for_merge(count), |(v1, v2)| { merge_sort::merge_from_end(v1, v2); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_forward", count), &count, |b, &count| {
            b.iter_batched_ref(|| vectors_for_merge(count), |(v1, v2)| { merge_sort::merge_forward(v1, v2); }, BatchSize::LargeInput)
        });
        group.bench_with_input(BenchmarkId::new("merge_reverse", count), &count, |b, &count| {
            b.iter_batched_ref(|| vectors_for_merge_reverse(count), |(v1, v2)| { merge_sort::merge_reverse(v1, v2); }, BatchSize::LargeInput)
        });
    }
    group.finish();
}

pub fn insertion_sort_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion_sort_compare");

    // for count in vec_powers(12, 1, 2) {
    for count in (1..=300).step_by(5) {
        group.bench_with_input(BenchmarkId::new("insertion_sort", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { insertion_sort(v); }, BatchSize::SmallInput)
        });
        group.bench_with_input(BenchmarkId::new("insertion_sort_small", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { insertion_sort::insertion_sort_small(v); }, BatchSize::SmallInput)
        });
        group.bench_with_input(BenchmarkId::new("insertion_sort_ptr", count), &count, |b, &count| {
            b.iter_batched_ref(|| vec_usize_shuffled(count), |v| { insertion_sort::insertion_sort_ptr(v); }, BatchSize::SmallInput)
        });
    }
    group.finish();
}

criterion_group!(benches,
    // vectors_for_merge_1_000,
    // vectors_for_merge_10_000,
    // merge_1_000,
    // merge_10_000,
    // merge_sort_100,
    // merge_sort_1_000,
    // merge_sort_10_000,
    // merge_sort_100_000,
    // merge_sort_1_000_000,
    // merge_sort_range,
    // merge_sort_vs_rust_sort,
    // sort_find_crossover
    // merge_sort_find_crossover_size
    // merge_sort_skip_merge
    // merge_compare
    // merge_direction
    // quicksort_find_crossover_point
    // quicksort_find_limit
    // quicksort_compare_pointer
    // quicksort_find_minimum_thread_size
    // quicksort_counter_overhead
    // quicksort_get_parent_overhead
    // quicksort_rayon_find_min_split_size,
    quicksort_rayon_find_min_thread_size,
    // insertion_sort_compare
    // sort_compare_shuffled
    // sort_compare_ordered,
    // sort_compare_reversed,
    );
criterion_main!(benches);

// From the main project folder run:
//   cargo +nighly bench
// for all benchmarks or:
//   cargo +nightly bench --bench sort_benchmark
// for just the above group.
