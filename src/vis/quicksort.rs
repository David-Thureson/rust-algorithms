#![allow(dead_code)]

use rayon::prelude::*;
use std::fmt::{self, Debug};

use crate::util;
use crate::sort::test_data;
use super::bubble_sort;
use super::model::*;

pub fn main() {
    // try_sort_small();
    try_log();
}

pub fn quicksort<T>(s: &mut [T], min_split_size: usize, max_threads: u8) -> RangeCall
    where T: Ord + Send + Debug
{
    quicksort_internal(s, 0, false, min_split_size, max_threads, "0")
}

fn quicksort_internal<T>(s: &mut [T], from: usize, is_new_thread: bool, min_split_size: usize, max_threads: u8, range_call_key: &str) -> RangeCall
    where T: Ord + Send + Debug
{
    let s_len = s.len();
    let mut range_call = RangeCall::new(&range_call_key, Actor::Quicksort, from, s_len, is_new_thread);
    let first_child_range_call_key = &format!("{}0", &range_call_key);
    if s_len > 1 {
        if s_len < min_split_size {
            range_call.add_child_call(bubble_sort::bubble_sort(s, from, first_child_range_call_key));
        } else {
            let (mid, partition_range_call) = partition(s, from, false, first_child_range_call_key);
            range_call.add_child_call(partition_range_call);
            let (lo, hi) = s.split_at_mut(mid);
            let (_, hi) = hi.split_at_mut(1);
            let hi_from = from + mid + 1;
            let second_child_range_call_key = &format!("{}1", &range_call_key);
            if max_threads == 1 {
                if lo.len() == 1 {
                    range_call.mark_final_one(0);
                } else {
                    if lo.len() < min_split_size {
                        range_call.add_child_call(bubble_sort::bubble_sort(lo, from, first_child_range_call_key));
                    } else {
                        range_call.add_child_call(quicksort_internal(lo, from, false, min_split_size, max_threads, first_child_range_call_key));
                    }
                }
                if hi.len() == 1 {
                    range_call.mark_final_one(mid + 1);
                } else {
                    if hi.len() < min_split_size {
                        range_call.add_child_call(bubble_sort::bubble_sort(hi, hi_from, second_child_range_call_key));
                    } else {
                        range_call.add_child_call(quicksort_internal(hi, hi_from, false, min_split_size, max_threads, second_child_range_call_key));
                    }
                }
            } else {
                range_call.add_child_calls(rayon::join(
                    || quicksort_internal(lo, from, true, min_split_size, max_threads / 2, first_child_range_call_key),
                    || quicksort_internal(hi, hi_from, true, min_split_size, max_threads / 2, second_child_range_call_key)
                ));
            }
        }
    }
    range_call.end();
    range_call
}

#[inline]
fn partition<T> (s: &mut [T], from: usize, is_new_thread: bool, range_call_key: &str) -> (usize, RangeCall)
    where T: Ord + Send + Debug
{
    let s_len = s.len();
    let mut range_call = RangeCall::new(&range_call_key, Actor::Partition, from, s_len, is_new_thread);
    let pivot_index = if s.len() <= 3 {
        1
    } else {
        // Median of the first, middle, and last elements.
        let mut pivots = [0, s.len() / 2, s.len() - 1];
        pivots.sort_unstable_by_key(|i| &s[*i]);
        pivots[1]
    };
    // s.partition_at_index(pivot);

    if pivot_index != 0 {
        s.swap(0, pivot_index);
        range_call.swap(0, pivot_index);
    }

    let mut i = 1;
    let mut j = s_len - 1;
    let mid;
    loop {
        while i < s_len && s[i] < s[0]{
            i +=1;
        }
        while j > 0 && s[j] > s[0] {
            j -= 1;
        }
        if i < j {
            s.swap(i, j);
            range_call.swap(i, j);
        }
        if i == j || j == i + 1 || i == j + 1 {
            mid = if i < j { i } else { j };
            if mid > 0 {
                s.swap(0, mid);
                range_call.swap(0, mid);
            }
            range_call.mark_final_one(mid);
            // assert_quicksort_invariant(&s, mid);
            break;
        } else {
            i += 1;
            j -= 1;
        }
    }

    range_call.end();
    (mid, range_call)
}

fn try_sort_small() {
    let min_split_size = 10;
    let max_threads = 4;
    for size in 1..= 100 {
        let mut v = test_data::vec_usize_shuffled(size);
        // bg!(&v);
        // quicksort_rayon_minimal(&mut v);
        quicksort(&mut v, min_split_size, max_threads);
        dbg!(&v);
        assert!(v.is_sorted());
    }
}

fn try_log() {
    let min_split_size = 5;
    let max_threads = 1;
    let size = 5;
    let mut v = test_data::vec_usize_shuffled(size);
    dbg!(&v);
    let include_detail_actions = true;
    quicksort(&mut v, min_split_size, max_threads)
        .try_log(include_detail_actions);
}
