use rayon::prelude::*;
use std::fmt::Debug;
use crate::sort::test_data;
use crate::sort::bubble_sort;

pub fn main() {
    try_sort_small();
}

pub fn quicksort_crossbeam_minimal<T>(s: &mut [T])
    where T: Ord + Send + Debug
{
    if s.len() > 1 {
        let mid = partition(s);
        let (lo, hi) = s.split_at_mut(mid);
        let _ = crossbeam::scope(|scope| {
            scope.spawn(move |_| quicksort_crossbeam_minimal(lo));
            scope.spawn(move |_| quicksort_crossbeam_minimal(hi));
        });
    }
}

pub fn quicksort_crossbeam<T>(s: &mut [T], min_split_size: u8, min_thread_size: u32)
    where T: Ord + Send + Debug
{
    let s_len = s.len();
    if s_len > 1 {
        if s_len < min_split_size as usize {
            bubble_sort::bubble_sort(s);
        } else {
            let mid = partition(s);
            let (lo, hi) = s.split_at_mut(mid);
            let min_thread_size_usize = min_thread_size as usize;
            if lo.len() < min_thread_size_usize && hi.len() < min_thread_size_usize {
                quicksort_crossbeam(lo, min_split_size, min_thread_size);
                quicksort_crossbeam(hi, min_split_size, min_thread_size);
            } else {
                let _ = crossbeam::scope(|scope| {
                    scope.spawn(move |_| quicksort_crossbeam(lo, min_split_size, min_thread_size));
                    scope.spawn(move |_| quicksort_crossbeam(hi, min_split_size, min_thread_size));
                });
            }
        }
    }
}

#[inline]
fn partition<T> (s: &mut [T]) -> usize
    where T: Ord + Send + Debug
{
    let pivot = if s.len() <= 3 {
        1
    } else {
        // Median of the first, middle, and last elements.
        let mut pivots = [0, s.len() / 2, s.len() - 1];
        pivots.sort_unstable_by_key(|i| &s[*i]);
        pivots[1]
    };
    s.partition_at_index(pivot);
    pivot
}

fn try_sort_small() {
    let min_split_size = 5;
    let min_thread_size = 15;
    for size in 1..= 60{
        let mut v = test_data::vec_usize_shuffled(size);
        dbg!(&v);
        // quicksort_crossbeam_minimal(&mut v);
        quicksort_crossbeam(&mut v, min_split_size, min_thread_size);
        dbg!(&v);
        assert!(v.is_sorted());
    }

}

