#![allow(dead_code)]

// Consider the various approaches in https://stackoverflow.com/questions/32750829/how-can-i-pass-a-reference-to-a-stack-variable-to-a-thread
// that use crossbeam scoped-threadpool, and rayon.

/*
use crate::*;
use super::*;
use super::test_data::*;
use counter::Counter;

use std::cmp;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use std::ptr;
use std::slice;
use std::sync::{Mutex, Arc};
use core::ptr::Unique;
use rand::prelude;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::borrow::Borrow;
use std::time::Instant;
*/

use std::fmt::Debug;
use std::sync::atomic;
use std::thread;
use rand;

use crate::sort::bubble_sort;
use crate::sort::test_data;
use rand::Rng;

pub fn main() {
    // try_small_vectors();
}

/*

#[derive(Copy, Clone)]
struct SafeParallelSettings {
    min_split_size: u8,
    min_find_partition_size: u8,
    min_thread_size: u32,
    ordering: atomic::Ordering,
}

pub fn quicksort_safe_parallel_vector<T> (
    v: Vec<T>,
    settings: SafeParallelSettings) -> Vec<T>
    where T: PartialOrd + Copy + Send + Debug
{
    quicksort_safe_parallel(&mut v[..], settings.min_split_size, settings.min_find_partition_size, settings.min_thread_size, settings.ordering);
    v
}

pub fn quicksort_safe_parallel<T>(
        s: &mut [T],
        min_split_size: u8,
        min_find_partition_size: u8,
        min_thread_size: u32,
        ordering: atomic::Ordering)
    where T: PartialOrd + Copy + Send + Debug
{
    let s_len = s.len();
    if s_len < min_split_size as usize {
        bubble_sort::bubble_sort(s);
        return;
    }
    let mut rng = rand::thread_rng();
    let settings = SafeParallelSettings {
        min_split_size,
        min_find_partition_size,
        min_thread_size,
        ordering,
    };
    quicksort_safe_parallel_internal(s, settings, &mut rng);
}

fn quicksort_safe_parallel_internal<T: PartialOrd + Debug> (s: &mut [T], settings: SafeParallelSettings, rng: &mut rand::prelude::ThreadRng)
    where T: PartialOrd + Copy + Send + Debug
{
    let slice_len = s.len();
    if slice_len >= settings.min_find_partition_size as usize {
        let slice_len_f64 = slice_len as f64;
        let i1: usize = (rng.gen::<f64>() * slice_len_f64) as usize;
        let i2: usize = (rng.gen::<f64>() * slice_len_f64) as usize;
        let i3: usize = (rng.gen::<f64>() * slice_len_f64) as usize;
        //bg!(i1, i2, i3);
        //bg!(&s[i1], &s[i2], &s[i3]);
        let partition_index =
            if s[i1] < s[i2] {
                if s[i1] < s[i3] {
                    if s[i2] < s[i3] {
                        i2
                    } else {
                        i3
                    }
                } else {
                    i1
                }
            } else {
                if s[i2] < s[i3] {
                    if s[i1] < s[i3] {
                        i1
                    } else {
                        i3
                    }
                } else {
                    i2
                }
            };
        //bg!(partition_index);
        if partition_index != 0 {
            s.swap(0, partition_index);
        }
    }

    let mut i = 1;
    let mut j = slice_len - 1;
    let mid;
    loop {
        while i < slice_len && s[i] < s[0]{
            i +=1;
        }
        while j > 0 && s[j] > s[0] {
            j -= 1;
        }
        if i < j {
            s.swap(i, j);
        }
        if i == j || j == i + 1 || i == j + 1 {
            mid = if i < j { i } else { j };
            if mid > 0 {
                s.swap(0, mid);
            }
            // assert_quicksort_invariant(&s, mid);
            break;
        } else {
            i += 1;
            j -= 1;
        }
    }

    let mut subslices = [&mut s[..mid], &mut s[mid+1..]];
    // Put the largest subslice first. This is the one that might get its own thread.
    subslices.sort_unstable_by_key(|x| 0 - x.len());

    // Deal with the larger subslice, whether that's the first or second one in the overall slice. It may get its
    // own thread.

    let subslice_0 = subslices[0];
    let subslice_len_0 = subslice_0.len();
    let handle = if subslice_len_0 < settings.min_split_size as usize {
        bubble_sort::bubble_sort(subslice_0);
        None
    } else if subslice_len_0 >= settings.min_thread_size as usize {
        let mut v = subslice_0.to_vec();
        Some(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            // let mut v = subslice_0.to_vec();
            // quicksort_safe_parallel_internal(&mut v[..], settings, &mut rng);
            quicksort_safe_parallel_vector(v, settings)
        }))
    } else {
        quicksort_safe_parallel_internal(subslice_0, settings, rng);
        None
    };

    // Deal with the smaller subslice. This is simpler since it will not get its own thread.
    let subslice_1 = subslices[1];
    let subslice_len_1 = subslice_1.len();
    if subslice_len_1 < settings.min_split_size as usize {
        bubble_sort::bubble_sort(subslice_1);
    } else {
        quicksort_safe_parallel_internal(subslice_1, settings, rng);
    }

    if let Some(handle) = handle {
        // let v: Vec<T> = handle.join().unwrap();

    }

}
*/

pub fn quicksort<T: PartialOrd + Debug> (s: &mut [T]) {
    const CROSSOVER_POINT: usize = 7;
    if s.len() <= CROSSOVER_POINT {
        bubble_sort(s);
        return;
    }
    let mut i = 1;
    let mut j = s.len() - 1;
    let mid;
    loop {
        while i < s.len() && s[i] < s[0]{
            i +=1;
        }
        while j > 0 && s[j] > s[0] {
            j -= 1;
        }
        if i < j {
            s.swap(i, j);
        }
        if i == j || j == i + 1 || i == j + 1 {
            mid = if i < j { i } else { j };
            if mid > 0 {
                s.swap(0, mid);
            }
            // assert_quicksort_invariant(&s, mid);
            break;
        } else {
            i += 1;
            j -= 1;
        }
    }
    quicksort(&mut s[..mid]);
    quicksort(&mut s[mid+1..]);
}

pub fn quicksort_rnd_3_with_limit<T: PartialOrd + Debug> (s: &mut [T], limit: usize) {
    let mut rng = rand::thread_rng();
    quicksort_rnd_3_internal_with_limit(s, &mut rng, limit);
}

fn quicksort_rnd_3_internal_with_limit<T: PartialOrd + Debug> (s: &mut [T], rng: &mut rand::prelude::ThreadRng, limit: usize) {
    let s_len = s.len();
    const CROSSOVER_POINT: usize = 7;
    if s_len <= CROSSOVER_POINT {
        bubble_sort(s);
        return;
    }

    if s_len > limit {
        let s_len_f64 = s_len as f64;
        let i1: usize = (rng.gen::<f64>() * s_len_f64) as usize;
        let i2: usize = (rng.gen::<f64>() * s_len_f64) as usize;
        let i3: usize = (rng.gen::<f64>() * s_len_f64) as usize;
        //bg!(i1, i2, i3);
        //bg!(&s[i1], &s[i2], &s[i3]);
        let partition_index =
            if s[i1] < s[i2] {
                if s[i1] < s[i3] {
                    if s[i2] < s[i3] {
                        i2
                    } else {
                        i3
                    }
                } else {
                    i1
                }
            } else {
                if s[i2] < s[i3] {
                    if s[i1] < s[i3] {
                        i1
                    } else {
                        i3
                    }
                } else {
                    i2
                }
            };
        //bg!(partition_index);
        if partition_index != 0 {
            s.swap(0, partition_index);
        }
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
        }
        if i == j || j == i + 1 || i == j + 1 {
            mid = if i < j { i } else { j };
            if mid > 0 {
                s.swap(0, mid);
            }
            // assert_quicksort_invariant(&s, mid);
            break;
        } else {
            i += 1;
            j -= 1;
        }
    }
    quicksort_rnd_3_internal_with_limit(&mut s[..mid], rng, limit);
    quicksort_rnd_3_internal_with_limit(&mut s[mid+1..], rng, limit);
}

pub fn quicksort_with_crossover<T: PartialOrd + Debug> (s: &mut [T], crossover_point: usize) {
    if s.len() <= crossover_point {
        bubble_sort(s);
        return;
    }
    let mut i = 1;
    let mut j = s.len() - 1;
    let mid;
    loop {
        //bg!("top of loop", i, j, &s);
        while i < s.len() && s[i] < s[0]{
            i +=1;
        }
        while j > 0 && s[j] > s[0] {
            j -= 1;
        }
        //bg!("before possible i, j swap", i, j);
        if i < j {
            s.swap(i, j);
            //bg!("after i, j swap", i, j, &s);
        }
        if i == j || j == i + 1 || i == j + 1 {
            mid = if i < j { i } else { j };
            if mid > 0 {
                //bg!("before lo swap", i, j, mid);
                s.swap(0, mid);
                //bg!("after lo swap", &s);
            }
            // assert_quicksort_invariant(&s, mid);
            break;
        } else {
            i += 1;
            j -= 1;
        }
    }
    quicksort_with_crossover(&mut s[..mid], crossover_point);
    //bg!("after low array sort", &s);
    quicksort_with_crossover(&mut s[mid+1..], crossover_point);
    //bg!("after high array sort", &s);
    //debug_assert!(s.is_sorted());
}

fn assert_quicksort_invariant<T: PartialOrd + Debug>(s: &[T], mid: usize) {
    if mid > 0 {
        for i in 0..mid {
            assert!(s[i] <= s[mid]);
        }
    }
    if mid < s.len() - 1 {
        for i in (mid + 1)..s.len() {
            assert!(s[i] >= s[mid]);
        }
    }
}

/*
fn try_small_vectors() {
    let size = 50;
    let min_split_size = 20;
    let min_find_partition_size = 30;
    let min_thread_size = 10_000_000;
    let ordering = atomic::Ordering::AcqRel;

    for i in 40..=40 {
        let mut v = test_data::vec_usize_shuffled(i);
        dbg!(&v);
        // quicksort_rnd_3_with_limit(&mut v[..], 20);
        quicksort_safe_parallel(&mut v[..], 20, min_find_partition_size, min_thread_size, ordering);
        dbg!(&v);
        assert!(&v.is_sorted());
    }

}
*/
/*
fn try_large_vector() {
    let size = 10_000;
    let mut v = vec_usize_shuffled(size);
    dbg!(&v.is_sorted());
    // merge_sort(&mut v);
    merge_sort_with_bubble(&mut v);
    dbg!(&v.is_sorted());
    assert!(&v.is_sorted());
}
*/

