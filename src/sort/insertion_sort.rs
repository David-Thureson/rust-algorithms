#![allow(dead_code)]

use super::test_data::*;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use std::ptr;

pub fn main() {
    // try_small_vectors();
    // try_large_vector();
    // try_insertion_sort_small();
    try_insertion_sort_ptr();
}

pub fn insertion_sort<T: Ord + Debug> (v: &mut Vec<T>) {
    let mut sorted: Vec<T> = Vec::with_capacity(v.len());
    while let Some(t) = v.pop() {
        let index = match sorted.binary_search(&t) {
            Ok(index) => index,
            Err(index) => index,
        };
        sorted.insert(index, t);
    }
    *v = sorted;
}

pub fn insertion_sort_small<T: PartialOrd + Debug> (v: &mut Vec<T>) {
    let mut sorted: Vec<T> = Vec::with_capacity(v.len());
    let mut sorted_len = 0;
    while let Some(t) = v.pop() {
        //bg!("top of while let", &v, &sorted, &sorted_len, &t);
        if sorted_len == 0 {
            sorted.push(t);
        } else {
            let mut i = 0;
            while i < sorted_len && t > sorted[i] {
                i += 1;
            }
            //bg!("after insert", i, &sorted);
            sorted.insert(i, t);
        }
        sorted_len += 1;
    }
    *v = sorted;
}

/*

            unsafe {
                if ptr::read(s_ptr.offset(j)) > ptr::read(s_ptr.offset(j + 1)) {
                    std::ptr::swap(s_ptr.offset(j), s_ptr.offset(j + 1));
                    did_swap = true;
                }
*/

pub fn insertion_sort_ptr<T: Ord + Debug> (v: &mut Vec<T>) {
    let mut sorted: Vec<T> = Vec::with_capacity(v.len());
    let s_ptr = sorted.as_mut_ptr();
    let mut sorted_len = 0 as isize;
    while let Some(t) = v.pop() {
        //bg!("top of while let", &sorted, &sorted_len, &t);
        let index;
        unsafe {
            match sorted_len {
                0 => {
                    //bg!("sorted is empty so push");
                    index = sorted_len;
                },
                1 => {
                    index = if t < ptr::read(s_ptr) {
                        //bg!("sorted has one item and the new item is smaller");
                        0
                    } else {
                        //bg!("sorted has one item and the new item is larger");
                        sorted_len
                    };
                },
                2 => {
                    index = if t < ptr::read(s_ptr) {
                        //bg!("sorted has two items and the new item is smaller than the first");
                        0
                    } else if t < ptr::read(s_ptr.offset(1)) {
                        //bg!("sorted has two items and the new item is smaller than the second");
                        1
                    } else {
                        //bg!("sorted has two items and the new item is larger than both");
                        sorted_len
                    };
                },
                _ => {
                    let mut left = 0isize;
                    let mut right = sorted_len;
                    loop {
                        match right - left {
                            1 => {
                                index = if t < ptr::read(s_ptr.offset(left)) {
                                    left
                                } else {
                                    right
                                };
                                break;
                            },
                            2 => {
                                index = if t < ptr::read(s_ptr.offset(left)) {
                                    left
                                } else if t < ptr::read(s_ptr.offset(left + 1)) {
                                    left + 1
                                } else {
                                    right
                                };
                                break;
                            },
                            _ => {
                                let mid = (left + right) / 2;
                                match t.cmp(&ptr::read(s_ptr.offset(mid))) {
                                    Ordering::Equal => {
                                        index = mid;
                                        break;
                                    },
                                    Ordering::Less => {
                                        right = mid;
                                    },
                                    Ordering::Greater => {
                                        left = mid;
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
        if index == sorted_len {
            sorted.push(t);
        } else {
            sorted.insert(index as usize, t);
        }
        sorted_len += 1;
        //bg!("after adding to sorted", &index, &sorted);
    }
    *v = sorted;
}

fn try_small_vectors() {
    for i in 1..=5 {
        let mut v = vec_usize_shuffled(i);
        dbg!(&v);
        insertion_sort(&mut v);
        dbg!(&v);
        assert!(&v.is_sorted());
    }
}

fn try_large_vector() {
    let size = 1000;
    let mut v = vec_usize_shuffled(size);
    dbg!(&v.is_sorted());
    insertion_sort(&mut v);
    assert_eq!(size, v.len());
    dbg!(&v.is_sorted());
    assert!(&v.is_sorted());
}

fn try_insertion_sort_small() {
    // let mut v = vec![1, 2];
    //bg!(&v);
    // insertion_sort_small(&mut v);
    //bg!(&v);

    for i in 1..=20 {
        let mut v = vec_usize_shuffled(i);
        // dbg!(&v);
        insertion_sort_small(&mut v);
        dbg!(&v);
        assert!(&v.is_sorted());
    }

}

fn try_insertion_sort_ptr() {
    // let mut v = vec![2, 1, 3];
    // dbg!(&v);
    // insertion_sort_ptr(&mut v);
    // dbg!(&v);


    for i in 1..=20 {
        let mut v = vec_usize_shuffled(i);
        dbg!(&v);
        insertion_sort_ptr(&mut v);
        dbg!(&v);
        assert!(&v.is_sorted());
    }

}