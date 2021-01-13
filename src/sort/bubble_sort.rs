#![allow(dead_code)]

use super::test_data::*;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use std::ptr;

pub fn main() {
    try_small_vectors();
    // try_large_vector();
    try_bubble_sort_ptr();
}

#[inline]
pub fn bubble_sort<T: PartialOrd + Debug> (v: &mut [T]) {
    let mut did_swap = true;
    for i in (1..v.len()).rev() {
        if !did_swap {
            return;
        }
        did_swap = false;
        for j in 0..i {
            if v[j] > v[j + 1] {
                v.swap(j, j + 1);
                did_swap = true;
            }
        }
    }
}


pub fn bubble_sort_ptr<T: PartialOrd + Debug> (s: &mut [T]) {
    let s_ptr = s.as_mut_ptr();
    let mut did_swap = true;
    for i in (1..s.len() as isize).rev() {
        if !did_swap {
            return;
        }
        did_swap = false;
        for j in 0..i {
            unsafe {
                if ptr::read(s_ptr.offset(j)) > ptr::read(s_ptr.offset(j + 1)) {
                    std::ptr::swap(s_ptr.offset(j), s_ptr.offset(j + 1));
                    did_swap = true;
                }
            }
        }
    }
}

fn try_small_vectors() {
    for i in 1..=100 {
        let mut v = vec_usize_shuffled(i);
        dbg!(&v);
        bubble_sort(&mut v);
        dbg!(&v);
        assert!(&v.is_sorted());
    }
}

fn try_large_vector() {
    let size = 100;
    let mut v = vec_usize_shuffled(size);
    dbg!(&v.is_sorted());
    bubble_sort(&mut v);
    assert_eq!(size, v.len());
    dbg!(&v.is_sorted());
    assert!(&v.is_sorted());
    dbg!(&v);
}

fn try_bubble_sort_ptr() {
    for i in 1..=100 {
        let mut v = vec_usize_shuffled(i);
        dbg!(&v);
        bubble_sort_ptr(&mut v[..]);
        dbg!(&v);
        assert!(&v.is_sorted());
    }
}
