#![allow(dead_code)]

use super::*;
use super::test_data::*;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use itertools::Itertools;
use std::convert::TryInto;

// use super::Sortable;

pub fn main() {
    // try_small_vectors();
    // try_merge();
    // try_large_vector();
    // try_merge_sort_test_only_no_merge();
    // try_merge_from_end();
    // try_merge_in_place();
    try_merge_in_place_as_sort();
    // try_merge_pointers();
    // try_merge_forward();
    // try_merge_reverse();
    // try_merge_sort_merge_in_place();
    // try_all_merges();
    // try_all_merge_sorts();
}

pub fn merge_sort<T: PartialOrd + Debug> (v: &mut Vec<T>) {
    let v_len = v.len();
    match v_len {
        1 => {},
        2 => {
            if v[0] > v[1] {
                v.swap(0, 1);
            }
        },
        _ => {
            let mid = v_len / 2;
            let mut v2 = v.split_off(mid);
            merge_sort(v);
            merge_sort(&mut v2);
            *v = merge(v, &mut v2);
        }
    }

}

pub fn merge_sort_with_bubble<T: PartialOrd + Debug> (v: &mut Vec<T>) {
    const CROSSOVER_POINT: usize = 10;
    let v_len = v.len();
    if v_len <= CROSSOVER_POINT {
        bubble_sort(v);
        return;
    }
    match v_len {
        1 => {},
        2 => {
            if v[0] > v[1] {
                v.swap(0, 1);
            }
        },
        _ => {
            let mid = v_len / 2;
            let mut v2 = v.split_off(mid);
            merge_sort_with_bubble(v);
            merge_sort_with_bubble(&mut v2);
            *v = merge(v, &mut v2);
        }
    }

}

pub fn merge_sort_with_bubble_set_crossover<T: PartialOrd + Debug> (v: &mut Vec<T>, crossover_point: usize) {
    let v_len = v.len();
    if v_len <= crossover_point {
        bubble_sort(v);
        return;
    }
    match v_len {
        1 => {},
        2 => {
            if v[0] > v[1] {
                v.swap(0, 1);
            }
        },
        _ => {
            let mid = v_len / 2;
            let mut v2 = v.split_off(mid);
            merge_sort_with_bubble_set_crossover(v, crossover_point);
            merge_sort_with_bubble_set_crossover(&mut v2, crossover_point);
            *v = merge(v, &mut v2);
        }
    }

}

pub fn merge_sort_skip_match<T: PartialOrd + Debug> (v: &mut Vec<T>) {
    const CROSSOVER_POINT: usize = 10;
    let v_len = v.len();
    if v_len <= CROSSOVER_POINT {
        bubble_sort(v);
        return;
    }
    let mid = v_len / 2;
    let mut v2 = v.split_off(mid);
    merge_sort_skip_match(v);
    merge_sort_skip_match(&mut v2);
    *v = merge(v, &mut v2);
}

pub fn merge_sort_merge_from_end<T: PartialOrd + Debug> (v: &mut Vec<T>) {
    // const CROSSOVER_POINT: usize = 10;
    const CROSSOVER_POINT: usize = 0;
    let v_len = v.len();
    if v_len <= CROSSOVER_POINT {
        bubble_sort(v);
        return;
    }
    match v_len {
        1 => {},
        2 => {
            if v[0] > v[1] {
                v.swap(0, 1);
            }
        },
        _ => {
            let mid = v_len / 2;
            let mut v2 = v.split_off(mid);
            merge_sort_merge_from_end(v);
            merge_sort_merge_from_end(&mut v2);
            *v = merge_from_end(v, &mut v2);
        }
    }
}

pub fn merge_sort_merge_in_place<T: Ord + Debug> (s: &mut [T]) {
    const CROSSOVER_POINT: usize = 10;
    if s.len() <= CROSSOVER_POINT {
        bubble_sort(s);
        return;
    }
    let mid = s.len() / 2;
    merge_sort_merge_in_place(&mut s[..mid]);
    merge_sort_merge_in_place(&mut s[mid..]);
    merge_in_place(s, mid);
}

pub fn merge_sort_test_only_no_merge<T: PartialOrd + Debug> (v: &mut Vec<T>) {
    const CROSSOVER_POINT: usize = 10;
    let v_len = v.len();
    if v_len <= CROSSOVER_POINT {
        bubble_sort(v);
        return;
    }
    match v_len {
        1 => {},
        2 => {
            if v[0] > v[1] {
                v.swap(0, 1);
            }
        },
        _ => {
            let mid = v_len / 2;
            let mut v2 = v.split_off(mid);
            merge_sort_test_only_no_merge(v);
            merge_sort_test_only_no_merge(&mut v2);
            //*v = merge(v, &mut v2);
            v.append(&mut v2);
        }
    }

}

pub fn merge<T: PartialOrd + Debug> (v1: &mut Vec<T>, v2: &mut Vec<T>) -> Vec<T> {
    //bg!("merge() start:", &v1, &v2);
    let v1_len = v1.len();
    let v2_len = v2.len();
    assert!(v1_len > 0);
    assert!(v2_len > 0);
    let mut v: Vec<T> = Vec::with_capacity(v1.len() + v2.len());
    let mut v1_next = v1.remove(0);
    let mut v2_next = v2.remove(0);
    loop {
        //bg!("loop start: ", &v, &v1_next, &v2_next);
        if v1_next.partial_cmp(&v2_next) == Some(Ordering::Less) {
            // First value is lower.
            if v1.len() > 0 {
                v.push(mem::replace(&mut v1_next, v1.remove(0)));
            } else {
                // The first vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // second vector.
                v.push(v1_next);
                v.push(v2_next);
                v.append(v2);
                //bg!("merge() end: filled from second vector: ", &v);
                return v;
            }
        } else {
            // Second value is equal or lower.
            if v2.len() > 0 {
                v.push(mem::replace(&mut v2_next, v2.remove(0)));
            } else {
                // The second vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // first vector.
                v.push(v2_next);
                v.push(v1_next);
                v.append(v1);
                //bg!("merge() end: filled from first vector: ", &v);
                return v;
            }
        }
    }
}

#[inline]
pub fn merge_from_end<T: PartialOrd + Debug> (v1: &mut Vec<T>, v2: &mut Vec<T>) -> Vec<T> {
    let v1_len = v1.len();
    let v2_len = v2.len();
    assert!(v1_len > 0);
    assert!(v2_len > 0);
    let mut v: Vec<T> = Vec::with_capacity(v1.len() + v2.len());
    let mut v1_next = v1.pop().unwrap();
    let mut v2_next = v2.pop().unwrap();
    loop {
        if v1_next.partial_cmp(&v2_next) == Some(Ordering::Greater) {
            // First value is higher.
            if v1.len() > 0 {
                v.push(mem::replace(&mut v1_next, v1.pop().unwrap()));
            } else {
                // The first vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // second vector.
                v.push(v1_next);
                v.push(v2_next);
                v2.reverse();
                v.append(v2);
                //bg!("merge() end: filled from second vector: ", &v);
                v.reverse();
                return v;
            }
        } else {
            // Second value is equal or higher.
            if v2.len() > 0 {
                v.push(mem::replace(&mut v2_next, v2.pop().unwrap()));
            } else {
                // The second vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // first vector.
                v.push(v2_next);
                v.push(v1_next);
                v1.reverse();
                v.append(v1);
                //bg!("merge() end: filled from first vector: ", &v);
                v.reverse();
                return v;
            }
        }
    }
}

pub fn merge_forward<T: PartialOrd + Debug> (v1: &mut Vec<T>, v2: &mut Vec<T>) -> Vec<T> {
    // Same as merge_from_end(), but leaves the result vector in reverse order.
    let v1_len = v1.len();
    let v2_len = v2.len();
    assert!(v1_len > 0);
    assert!(v2_len > 0);
    let mut v: Vec<T> = Vec::with_capacity(v1.len() + v2.len());
    let mut v1_next = v1.pop().unwrap();
    let mut v2_next = v2.pop().unwrap();
    loop {
        if v1_next.partial_cmp(&v2_next) == Some(Ordering::Greater) {
            // First value is higher.
            if v1.len() > 0 {
                v.push(mem::replace(&mut v1_next, v1.pop().unwrap()));
            } else {
                // The first vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // second vector.
                v.push(v1_next);
                v.push(v2_next);
                v2.reverse();
                v.append(v2);
                //bg!("merge() end: filled from second vector: ", &v);
                return v;
            }
        } else {
            // Second value is equal or higher.
            if v2.len() > 0 {
                v.push(mem::replace(&mut v2_next, v2.pop().unwrap()));
            } else {
                // The second vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // first vector.
                v.push(v2_next);
                v.push(v1_next);
                v1.reverse();
                v.append(v1);
                //bg!("merge() end: filled from first vector: ", &v);
                return v;
            }
        }
    }
}

pub fn merge_reverse<T: PartialOrd + Debug> (v1: &mut Vec<T>, v2: &mut Vec<T>) -> Vec<T> {
    // Same as merge_forward() except that the argument vectors are given in reverse order and the
    // return vector is in forward order.
    let v1_len = v1.len();
    let v2_len = v2.len();
    assert!(v1_len > 0);
    assert!(v2_len > 0);
    let mut v: Vec<T> = Vec::with_capacity(v1.len() + v2.len());
    let mut v1_next = v1.pop().unwrap();
    let mut v2_next = v2.pop().unwrap();
    loop {
        //bg!("top of loop", &v1, &v2, &v1_next, &v2_next);
        if v1_next.partial_cmp(&v2_next) == Some(Ordering::Less) {
            // First value is lower.
            //bg!("first value is lower");
            if v1.len() > 0 {
                v.push(mem::replace(&mut v1_next, v1.pop().unwrap()));
                //bg!(&v1, &v1_next, &v);
            } else {
                // The first vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // second vector.
                v.push(v1_next);
                v.push(v2_next);
                v2.reverse();
                v.append(v2);
                //bg!("end: filled from second vector", &v);
                return v;
            }
        } else {
            // Second value is equal or lower.
            //bg!("second value is equal or lower");
            if v2.len() > 0 {
                v.push(mem::replace(&mut v2_next, v2.pop().unwrap()));
                //bg!(&v2, &v2_next, &v);
            } else {
                // The second vector is empty so put the two next values we already have into the
                // result vector (in the proper order) and take all of the remaining items from the
                // first vector.
                v.push(v2_next);
                v.push(v1_next);
                v1.reverse();
                v.append(v1);
                //bg!("end: filled from second vector", &v);
                return v;
            }
        }
    }
}

#[inline]
pub fn merge_in_place<T: Ord + Debug> (s: &mut [T], mid: usize) {
    let s_len = s.len();
    //debug_assert!(s_len >= 3);
    //debug_assert!(mid > 0);
    //debug_assert!(mid < s_len);
    //debug_assert!(s[..mid].is_sorted());
    //debug_assert!(s[mid..].is_sorted());
    let mut next_second_index = mid;
    while next_second_index < s_len {
        let insertion_index = match s[..next_second_index].binary_search(&s[next_second_index]) {
            Ok(index) => index,
            Err(index) => index,
        };
        if insertion_index < next_second_index {
            s[insertion_index..next_second_index + 1].rotate_right(1);
        }
        next_second_index += 1;
        //debug_assert!(s[..next_second_index].is_sorted());
        //debug_assert!(s[next_second_index..].is_sorted());
    }
    //debug_assert!(s.is_sorted())
}

#[inline]
pub fn merge_in_place_track_start<T: Ord + Debug> (s: &mut [T], mid: usize) {
    let s_len = s.len();
    //debug_assert!(s_len >= 3);
    //debug_assert!(mid > 0);
    //debug_assert!(mid < s_len);
    //debug_assert!(s[..mid].is_sorted());
    //debug_assert!(s[mid..].is_sorted());
    let mut next_second_index = mid;
    let mut search_start_index = 0;
    while next_second_index < s_len {
        let insertion_index = match s[search_start_index..next_second_index].binary_search(&s[next_second_index]) {
            Ok(index) => index,
            Err(index) => index,
        } + search_start_index;
        if insertion_index < next_second_index {
            s[insertion_index..next_second_index + 1].rotate_right(1);
        }
        search_start_index = insertion_index + 1;
        next_second_index += 1;
        //debug_assert!(s[..next_second_index].is_sorted());
        //debug_assert!(s[next_second_index..].is_sorted());
    }
    //debug_assert!(s.is_sorted())
}

/*
    let start = Instant::now();
    let a_ptr: *const usize = a.as_ptr();
    let mut sum = 0;
    let mut i: isize = 0;
    let a_len = a_len as isize;
    unsafe {
        while i < a_len {
            sum += *a_ptr.offset(i);
            i += 1;
        }
    }

*/

pub fn merge_pointers<T: Ord + Debug> (s: &mut [T], mid: usize) {
    let s_len = s.len();
    debug_assert!(s_len >= 3);
    debug_assert!(mid > 0);
    debug_assert!(mid < s_len);
    debug_assert!(s[..mid].is_sorted());
    debug_assert!(s[mid..].is_sorted());
    let mut next_second_index = mid;
    let mut search_start_index = 0;
    let s_ptr = s.as_mut_ptr();
    while next_second_index < s_len {
        let insertion_index = match s[search_start_index..next_second_index].binary_search(&s[next_second_index]) {
            Ok(index) => index,
            Err(index) => index,
        } + search_start_index;
        if insertion_index < next_second_index {
            // s[insertion_index..next_second_index + 1].rotate_right(1);
            unsafe {
                let next_second_isize = next_second_index as isize;
                let t = std::ptr::read(s_ptr.offset(next_second_isize));
                let src = s_ptr.offset(insertion_index.try_into().unwrap());
                let dst = src.offset(1);
                let count = next_second_index - insertion_index;
                std::ptr::copy(src, dst, count);
                // Write the T value to the position where we started the copy. That's why the
                // value for the dst parameter is src.
                std::ptr::write(src, t);
            }
        }
        search_start_index = insertion_index + 1;
        next_second_index += 1;
        debug_assert!(s[..next_second_index].is_sorted());
        debug_assert!(s[next_second_index..].is_sorted());
    }

        /*
        let s_ptr = s.as_ptr();
        unsafe {
            let len_ptr = s_ptr.offset(s_len.try_into().unwrap());
            let mut next_second_ptr = s_ptr.offset(mid.try_into().unwrap());
            let mut ins_mid: *const T;
            while next_second_ptr < len_ptr {
                let ins_mid =
            let insertion_index = match s[..next_second_index].binary_search(&s[next_second_index]) {
                Ok(index) => index,
                Err(index) => index,
            };
    }
    */
    debug_assert!(s.is_sorted())
}
/*
pub fn merge_itertools<T: PartialOrd + Debug> (v1: &mut Vec<T>, v2: &mut Vec<T>) -> Vec<T> {
    let merge = v1.iter().merge(v2.iter());
    merge.into_iter().drain()
}
*/

fn try_small_vectors() {
    /*
    let mut v = vec![3, 2, 1];
    dbg!(&v);
    merge_sort(&mut v);
    dbg!(&v);
    */
    for i in 1..=10 {
        let mut v = vec_usize_shuffled(i);
        dbg!(&v);
        merge_sort(&mut v);
        dbg!(&v);
        assert!(&v.is_sorted());
    }

}

fn try_merge_sort_merge_in_place() {
    for i in 1..=10 {
        let mut v = vec_usize_shuffled(i);
        dbg!(&v);
        merge_sort_merge_in_place(&mut v[..]);
        dbg!(&v);
        assert!(&v.is_sorted());
    }

}

fn try_large_vector() {
    let size = 10_000;
    let mut v = vec_usize_shuffled(size);
    dbg!(&v.is_sorted());
    // merge_sort(&mut v);
    merge_sort_with_bubble(&mut v);
    dbg!(&v.is_sorted());
    assert!(&v.is_sorted());
}

fn try_merge_sort_test_only_no_merge() {
    let size = 1_000;
    let mut v = vec_usize_shuffled(size);
    dbg!(&v.is_sorted());
    merge_sort_test_only_no_merge(&mut v);
    dbg!(&v.is_sorted());
    assert!(!&v.is_sorted());
}

fn try_merge_from_end() {
    for i in 2..=4 {
        let (mut v1, mut v2) = vectors_for_merge(i);
        dbg!(&v1, &v2);
        dbg!(merge_from_end(&mut v1, &mut v2));
    }

}

fn try_merge_in_place() {
    for i in 8..=20 {
        let (mut v, mid) = vector_for_merge_in_place(i);
        merge_in_place(&mut v[..], mid);
        dbg!(&v.is_sorted(), &v);
        assert!(&v.is_sorted());
    }

}

fn try_merge_in_place_as_sort() {
    for i in 8..=20 {
        let mut v = vec_usize_shuffled(i);
        merge_in_place(&mut v[..], 1);
        dbg!(&v.is_sorted(), &v);
        assert!(&v.is_sorted());
    }

}

fn try_merge_pointers() {
    for i in 3..=40 {
        let (mut v, mid) = vector_for_merge_in_place(i);
        merge_pointers(&mut v[..], mid);
        dbg!(&v.is_sorted(), &v);
        assert!(&v.is_sorted());
    }

}

fn try_merge_forward() {
    for i in 3..=40 {
        let (mut v1, mut v2) = vectors_for_merge(i);
        dbg!(&v1, &v2);
        let mut v = merge_forward(&mut v1, &mut v2);
        dbg!(&v);
        v.reverse();
        assert!(&v.is_sorted());
    }
}

fn try_merge_reverse() {
    // let mut v1 = vec![4, 3, 2];
    // let mut v2 = vec![7, 6, 5, 1];
    // dbg!(&v1, &v2);
    // let v = merge_reverse(&mut v1, &mut v2);
    // dbg!(&v);

    for i in 3..=40 {
        let (mut v1, mut v2) = vectors_for_merge_reverse(i);
        dbg!(&v1, &v2);
        let v = merge_reverse(&mut v1, &mut v2);
        dbg!(&v);
        assert!(&v.is_sorted());
    }
}

fn try_all_merges() {
    dbg!("try_all_merges()");
    let size = 1_000;
    for func in [merge, merge_from_end].iter() {
        let (mut v1, mut v2) = vectors_for_merge(size);
        let v = func(&mut v1, &mut v2);
        assert!(&v.is_sorted());
        assert_eq!(size, v.len());
        assert_eq!(0, v1.len());
        assert_eq!(0, v2.len());
    }
    for func in [merge_in_place, merge_in_place_track_start, merge_pointers].iter() {
        let (mut v, mid) = vector_for_merge_in_place(size);
        func(&mut v[..], mid);
        assert!(&v.is_sorted());
        assert_eq!(size, v.len());
    }
}

fn try_all_merge_sorts() {
    dbg!("try_all_merge_sorts()");
    let size = 1_000;
    for sort_func in [merge_sort, merge_sort_with_bubble, merge_sort_skip_match, merge_sort_merge_from_end].iter() {
        let mut v = vec_usize_shuffled(size);
        assert!(!&v.is_sorted());
        sort_func(& mut v);
        assert!(&v.is_sorted());
    }
    let mut v = vec_usize_shuffled(size);
    assert!(!&v.is_sorted());
    merge_sort_merge_in_place(& mut v[..]);
    assert!(&v.is_sorted());
}

fn try_merge() {
    dbg!(merge(&mut vec![2], &mut vec![1]));
    dbg!(merge(&mut vec![3], &mut vec![1, 2]));
    dbg!(merge(&mut vec![2, 3], &mut vec![1, 4]));
    dbg!(merge(&mut vec![1, 5, 6, 7, 11], &mut vec![2, 3, 4, 8, 9, 10]));
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    /*
    #[bench]
    fn bench_merge_10(b: &mut Bencher) {
        b.iter(|| {
            let (mut v1, mut v2) = vectors_for_merge(10);
            dbg!(&v1, &v2);
            merge(&mut v1, &mut v2);
            // Return a value from the closure so the processing is not optimized out by the
            // compiler.
            v1
        });
    }
    */
}
