#![allow(dead_code)]

use rayon::prelude::*;
use std::fmt::Debug;
use crate::sort::test_data;
use crate::sort::bubble_sort;
use crate::sort::merge_sort;
use std::cmp::min;

pub fn main() {
    // try_sort_specific_case();
    try_sort_small();
}

pub fn merge_sort_loop<T> (s: &mut [T], min_split_size: u8, max_threads: u8)
    where T: Ord + Send + Debug
{
    let s_len = s.len();
    match s_len {
        1 => {},
        2 => {
            if s[0] > s[1] {
                s.swap(0, 1);
            }
            //ebug_assert!(s.is_sorted());
        },
        _ => {
            // if s_len < min_split_size as usize {
            //    bubble_sort::bubble_sort(s);
            //    //ebug_assert!(s.is_sorted());
            //    return;
            //}
            if max_threads >= 2 {
                let mid = s_len / 2;
                //bg!(&s, &mid);
                {
                    let (lo, hi) = s.split_at_mut(mid);
                    //bg!(&lo, &hi);
                    //bg!("rayon::join() -> Start");
                    rayon::join(
                        || merge_sort_loop(lo, min_split_size, max_threads / 2),
                        || merge_sort_loop(hi, min_split_size, max_threads / 2)
                    );
                    //bg!("rayon::join() -> Done");
                    //bg!(&lo, &hi);
                }
                merge_sort::merge_in_place_track_start(s, mid);
                //dbg!(&s);
            } else {
                let mut subslice_len = 2;
                while subslice_len < s_len {
                    if subslice_len == 2 {
                        for i in (0..s_len).step_by(2) {
                            if i + 1 < s_len {
                                if s[i] > s[i + 1] {
                                    s.swap(i, i + 1);
                                }
                            }
                        }
                    }
                    let next_subslice_len = subslice_len * 2;
                    for i in (0..s_len - 1).step_by(next_subslice_len) {
                        let end_index = min(i + next_subslice_len, s_len);
                        let merge_slice = &mut s[i..end_index];
                        merge_sort::merge_in_place_track_start(merge_slice, subslice_len);
                        //ebug_assert!(merge_slice.is_sorted());
                    }
                    subslice_len = next_subslice_len;
                }
            }
            //ebug_assert!(s.is_sorted());
        }
    }

}

pub fn merge_sort_loop_vec<T> (s: &mut Vec<T>, min_split_size: u8, max_threads: u8)
    where T: Ord + Send + Debug
{
    let s_len = s.len();
    match s_len {
        1 => {}
        2 => {
            if s[0] > s[1] {
                s.swap(0, 1);
            }
            //ebug_assert!(s.is_sorted());
        },
        _ => {
            if max_threads >= 2 {
                let mid = s_len / 2;
                //bg!(&s, &mid);
                let mut hi = s.split_off(mid);
                //bg!(&lo, &hi);
                //bg!("rayon::join() -> Start");
                rayon::join(
                    || merge_sort_loop(s, min_split_size, max_threads / 2),
                    || merge_sort_loop(&mut hi, min_split_size, max_threads / 2)
                );
                //bg!("rayon::join() -> Done");
                //bg!(&lo, &hi);
                *s = merge_sort::merge_from_end(s, &mut hi);
                //dbg!(&s);
            } else {
                let mut subslice_len = 2;
                while subslice_len < s_len {
                    if subslice_len == 2 {
                        for i in (0..s_len).step_by(2) {
                            if i + 1 < s_len {
                                if s[i] > s[i + 1] {
                                    s.swap(i, i + 1);
                                }
                            }
                        }
                    }
                    let next_subslice_len = subslice_len * 2;
                    for i in (0..s_len - 1).step_by(next_subslice_len) {
                        let end_index = min(i + next_subslice_len, s_len);
                        let merge_slice = &mut s[i..end_index];
                        merge_sort::merge_in_place_track_start(merge_slice, subslice_len);
                        //ebug_assert!(merge_slice.is_sorted());
                    }
                    subslice_len = next_subslice_len;
                }
            }
            //ebug_assert!(s.is_sorted());
        }
    }

}

fn try_sort_specific_case() {
    let min_split_size = 0;
    let max_threads = 1;
    let mut v = vec![1, 3, 2];
    merge_sort_loop(&mut v, min_split_size, max_threads);
    dbg!(&v);
}

fn try_sort_small() {
    let min_split_size = 5;
    let max_threads = 8;
    for size in 100..=100 {
        let mut v = test_data::vec_usize_shuffled(size);
        dbg!(&v);
        // merge_sort_loop(&mut v, min_split_size, max_threads);
        merge_sort_loop_vec(&mut v, min_split_size, max_threads);
        dbg!(&v);
        assert!(v.is_sorted());
    }
}

