#![allow(dead_code)]

use rand::prelude::*;

pub fn main() {
    try_vec_usize_shuffled();
    try_vec_usize_ordered();
    try_vec_usize_reversed();
    // try_vec_powers();
}

pub fn vec_usize_shuffled(size: usize) -> Vec<usize> {
    assert!(size > 0);
    let mut nums: Vec<usize> = (1..=size).collect();
    let mut rng = rand::thread_rng();
    nums.shuffle(&mut rng);
    nums
}

pub fn vec_usize_ordered(size: usize) -> Vec<usize> {
    assert!(size > 0);
    (1..=size).collect()
}

pub fn vec_usize_reversed(size: usize) -> Vec<usize> {
    assert!(size > 0);
    (1..=size).rev().collect()
}

pub fn vectors_for_merge(size: usize) -> (Vec<usize>, Vec<usize>) {
    assert!(size >= 2);
    let mut v1 = vec_usize_shuffled(size);
    let mid = if size <= 10 {
        size / 2
    } else {
        let size_f64 = size as f64;
        let mut rng = rand::thread_rng();
        let random: f64 = rng.gen();
        let start_64 = size_f64 * 0.25f64;
        let inc_64: f64 = size_f64 * random * 0.5f64;
        let mid_f64: f64 = start_64 + inc_64;
        mid_f64 as usize
    };
    let mut v2 = v1.split_off(mid);
    v1.sort_unstable();
    v2.sort_unstable();
    assert!(v1.len() > 0);
    assert!(v2.len() > 0);
    assert_eq!(size, v1.len() + v2.len());
    (v1, v2)
}

pub fn vectors_for_merge_reverse(size: usize) -> (Vec<usize>, Vec<usize>) {
    let (mut v1, mut v2) = vectors_for_merge(size);
    v1.reverse();
    v2.reverse();
    (v1, v2)
}

pub fn vector_for_merge_in_place(size: usize) -> (Vec<usize>, usize) {
    debug_assert!(size >= 3);
    let (mut v1, mut v2) = vectors_for_merge(size);
    let mid = v1.len();
    v1.append(&mut v2);
    debug_assert_eq!(size, v1.len());
    debug_assert!(mid > 0);
    debug_assert!(mid < v1.len());
    debug_assert!(v1[..mid].is_sorted());
    debug_assert!(v1[mid..].is_sorted());
    (v1, mid)
}

pub fn vec_powers<T>(count: u8, start: T, mult: T) -> Vec<T>
    where T: Copy + std::ops::MulAssign
{
    let mut v = vec![];
    let mut i = start;
    while v.len() < count as usize {
        v.push(i);
        i *= mult;
    }
    v
}

fn try_vec_usize_shuffled() {
    for i in 1..=10 {
        dbg!(vec_usize_shuffled(i));
    }
}

fn try_vec_usize_ordered() {
    for i in 1..=10 {
        dbg!(vec_usize_ordered(i));
    }
}

fn try_vec_usize_reversed() {
    for i in 1..=10 {
        dbg!(vec_usize_reversed(i));
    }
}

fn try_vec_powers() {
    dbg!(vec_powers(10, 1, 2));
    dbg!(vec_powers(5, 100, 10));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_unique_values<T: Ord>(v: &mut Vec<T>) {
        let v_len = v.len();
        v.sort_unstable();
        // let (first, second) = v.as_ref().partition_dedup::<([T], [T])>();
        let (first, second) = v.partition_dedup();
        assert_eq!(v_len, first.len(), "The first slice should contain all of the items from the vector.");
        assert_eq!(0, second.len(), "The second slice should be empty.");
    }

    # [test]
    fn test_vec_usize() {
        let mut v = vec_usize_shuffled(10);
        assert_eq!(10, v.len());
        assert_unique_values(&mut v);
    }

    # [test]
    # [should_panic]
    fn test_vec_usize_zero() {
        let mut _v = vec_usize_shuffled(0);
    }

}
