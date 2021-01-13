#![allow(dead_code)]

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
use std::thread;
use core::ptr::Unique;
use rand::prelude;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::borrow::Borrow;
use std::time::Instant;

pub fn main() {
    try_ptr_parallel();
}

pub fn quicksort_rnd_3_ptr_with_limit<T: PartialOrd + Debug> (s: &mut [T], limit: usize) {
    let mut rng = rand::thread_rng();
    quicksort_rnd_3_ptr_internal_with_limit(s, &mut rng, limit);
}

fn quicksort_rnd_3_ptr_internal_with_limit<T: PartialOrd + Debug> (s: &mut [T], rng: &mut ThreadRng, limit: usize) {
    let s_len: isize = s.len() as isize;
    // const CROSSOVER_POINT: isize = 7;
    const CROSSOVER_POINT: isize = 0;
    if s_len <= CROSSOVER_POINT {
        bubble_sort(s);
        return;
    }

    let s_ptr = s.as_mut_ptr();

    if s_len > limit as isize {
        let s_len_f64 = s_len as f64;
        let i1= (rng.gen::<f64>() * s_len_f64) as isize;
        let i2= (rng.gen::<f64>() * s_len_f64) as isize;
        let i3= (rng.gen::<f64>() * s_len_f64) as isize;
        //bg!(i1, i2, i3);
        //bg!(&s[i1], &s[i2], &s[i3]);
        let t1;
        let t2;
        let t3;
        unsafe {
            // t1 = *s_ptr.offset(i1);
            t1 = std::ptr::read(s_ptr.offset(i1));
            t2 = std::ptr::read(s_ptr.offset(i2));
            t3 = std::ptr::read(s_ptr.offset(i3));
        }
        let partition_index =
            if t1 < t2 {
                if t1 < t3 {
                    if t2 < t3 {
                        i2
                    } else {
                        i3
                    }
                } else {
                    i1
                }
            } else {
                if t2 < t3 {
                    if t1 < t3 {
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
            unsafe {
                std::ptr::swap(s_ptr, s_ptr.offset(partition_index));
            }
        }
    }

    let mut i: isize = 1;
    let mut j: isize = s_len - 1;
    let mid;
    loop {
        unsafe {
            let t_partition = std::ptr::read(s_ptr);
            while i < s_len && std::ptr::read(s_ptr.offset(i)) < t_partition {
                i += 1;
            }
            while j > 0 && std::ptr::read(s_ptr.offset(j)) > s[0] {
                j -= 1;
            }
            if i < j {
                std::ptr::swap(s_ptr.offset(i), s_ptr.offset(j));
            }
            if i == j || j == i + 1 || i == j + 1 {
                mid = if i < j { i } else { j };
                if mid > 0 {
                    std::ptr::swap(s_ptr, s_ptr.offset(mid));
                }
                // assert_quicksort_invariant(&s, mid);
                break;
            } else {
                i += 1;
                j -= 1;
            }
        }
    }
    let mid = mid as usize;
    quicksort_rnd_3_ptr_internal_with_limit(&mut s[..mid], rng, limit);
    quicksort_rnd_3_ptr_internal_with_limit(&mut s[mid+1..], rng, limit);
}

#[derive(Debug)]
struct Subslice<T: PartialOrd + Send + Debug> {
    // s_ptr: *mut T<'a>,
    // s_ptr: *'a mut T,
    // s_ptr: (*mut T)<'a>,
    // s_ptr: *mut T,
    // s_ptr_unique: isize,
    s_ptr_unique: Unique<T>,
    s_len: isize,
    thread_min_size: isize,
    overall_start_index: usize,
    // counter: Option<counter::Counter<SliceCounterItem>>,
}

// impl<T: PartialOrd + Debug> Clone for Subslice<T> {}

// unsafe impl<T: PartialOrd + Debug> Send for Subslice<T> {}

// unsafe impl<T: PartialOrd + Debug> Sync for Subslice<T> {}

pub fn quicksort_parallel_ptr<T: 'static + PartialOrd + Send + Debug> (
    s: &mut [T],
    thread_min_fraction: f64,
    thread_min_size: usize,
    fill_counter: bool,
    crossover_point: usize)
    -> Option<Counter<SliceCounterItem>>
{
    // const NEW_THREAD_MIN_FRACTION: f64 = 0.10;
    let len_isize = s.len() as isize;
    let calc_thread_min_size = (thread_min_fraction as f64 * len_isize as f64) as isize;
    let calc_thread_min_size = cmp::max(calc_thread_min_size, thread_min_size as isize);
    let subslice = Subslice {
        s_ptr_unique: unsafe { Unique::new_unchecked(s.as_mut_ptr()) },
        s_len: len_isize,
        thread_min_size: calc_thread_min_size,
        // counter: counter::Counter::new(),
        overall_start_index: 0,
    };
    let counter = if fill_counter {
        Some(Arc::new(Mutex::new(Counter::new())))
    } else {
        None
    };
    let one_counter = counter.as_ref().map(|x| Arc::clone(&x));
    quicksort_parallel_ptr_internal(subslice, one_counter, None, false, crossover_point);
    // counter.map(|arc| arc.into_inner().unwrap().clone())
    counter.map(|arc| {
        let m: &Mutex<Counter<SliceCounterItem>> = arc.borrow();
        m.lock().unwrap().clone()
    })
}

fn quicksort_parallel_ptr_internal<T: 'static + PartialOrd + Send + Debug> (
    subslice: Subslice<T>,
    counter: Option<Arc<Mutex<Counter<SliceCounterItem>>>>,
    counter_parent_index: Option<usize>,
    is_new_thread: bool,
    crossover_point: usize)
{
    // This is a variation on the pointer implementation that takes a pointer and a length instead
    // of a slice for convenience in sending sections from the original slice to other threads.
    // const CROSSOVER_POINT: isize = 11;
    // let CROSSOVER_POINT = crossover_point as isize;
    const LIMIT: isize = 15;

    // let Subslice { s_ptr, s_len, thread_min_size } = subslice;
    let s_ptr = subslice.s_ptr_unique.as_ptr();
    let s_len = subslice.s_len;
    let thread_min_size = subslice.thread_min_size;
    let overall_start_index = subslice.overall_start_index;

    let counter_index = counter.as_ref().map(|ct| {
        ct.lock().unwrap().start(
            s_len as usize,
            Some(SliceCounterItem {
                start_index: overall_start_index,
                end_index: overall_start_index + s_len as usize,
                method: Some(if s_len <= crossover_point as isize { "bubble sort".to_string() } else { "quicksort".to_string() }),
            }),
            is_new_thread,
            counter_parent_index
        )
    });

    if s_len <= crossover_point as isize {
        let s: &mut [T] = unsafe { slice::from_raw_parts_mut(s_ptr, s_len as usize) };
        bubble_sort(s);
        if let Some(ct) = counter {
            ct.lock().unwrap().end(counter_index.unwrap());
        }
        return;
    }

    if s_len > LIMIT {
        let s_len_f64 = s_len as f64;
        let mut rng = rand::thread_rng();
        let i1= (rng.gen::<f64>() * s_len_f64) as isize;
        let i2= (rng.gen::<f64>() * s_len_f64) as isize;
        let i3= (rng.gen::<f64>() * s_len_f64) as isize;
        //bg!(i1, i2, i3);
        //bg!(&s[i1], &s[i2], &s[i3]);
        let t1;
        let t2;
        let t3;
        unsafe {
            // t1 = *s_ptr.offset(i1);
            t1 = std::ptr::read(s_ptr.offset(i1));
            t2 = std::ptr::read(s_ptr.offset(i2));
            t3 = std::ptr::read(s_ptr.offset(i3));
        }
        let partition_index =
            if t1 < t2 {
                if t1 < t3 {
                    if t2 < t3 {
                        i2
                    } else {
                        i3
                    }
                } else {
                    i1
                }
            } else {
                if t2 < t3 {
                    if t1 < t3 {
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
            unsafe {
                std::ptr::swap(s_ptr, s_ptr.offset(partition_index));
            }
        }
    }

    let mut i: isize = 1;
    let mut j: isize = s_len - 1;
    let mid;
    loop {
        unsafe {
            let t_partition = std::ptr::read(s_ptr);
            while i < s_len && std::ptr::read(s_ptr.offset(i)) < t_partition {
                i += 1;
            }
            while j > 0 && std::ptr::read(s_ptr.offset(j)) > t_partition {
                j -= 1;
            }
            if i < j {
                std::ptr::swap(s_ptr.offset(i), s_ptr.offset(j));
            }
            if i == j || j == i + 1 || i == j + 1 {
                mid = if i < j { i } else { j };
                if mid > 0 {
                    std::ptr::swap(s_ptr, s_ptr.offset(mid));
                }
                // assert_quicksort_invariant(&s, mid);
                break;
            } else {
                i += 1;
                j -= 1;
            }
        }
    }

    // Sort the subslices.

    let mut subslices = vec![
        Subslice {
            s_ptr_unique: unsafe { ptr::Unique::new_unchecked(s_ptr) },
            s_len: mid,
            thread_min_size,
            overall_start_index: overall_start_index,
        },
        Subslice {
            s_ptr_unique: unsafe { ptr::Unique::new_unchecked(s_ptr.offset(mid + 1)) },
            s_len: (s_len - mid) - 1,
            thread_min_size,
            overall_start_index: overall_start_index + mid as usize + 1,
        }];

    // Put the largest subslice first.
    subslices.sort_unstable_by_key(|x| 0 - x.s_len);

    // Sort the larger of the subslices.
    let mut handle = None;
    let one_subslice = subslices.remove(0);
    let one_counter = counter.as_ref().map(|x| Arc::clone(&x));
    if one_subslice.s_len >= thread_min_size {
        // The largest subslice is long enough to get its own thread.
        //bg!("Starting a thread for {?:}", &one_subslice);
        handle = Some(thread::spawn(move || {
            quicksort_parallel_ptr_internal(one_subslice, one_counter,counter_index, true, crossover_point);
        }));
    } else {
        quicksort_parallel_ptr_internal(one_subslice, one_counter, counter_index, false, crossover_point);
    }

    // Sort the smaller of the subslices.
    let one_subslice = subslices.remove(0);
    if one_subslice.s_len > 0 {
        let one_counter = counter.as_ref().map(|x| Arc::clone(&x));
        quicksort_parallel_ptr_internal(one_subslice, one_counter, counter_index, false, crossover_point);
    }

    if let Some(handle) = handle {
        handle.join().unwrap();
    }

    if let Some(ct) = counter {
        ct.lock().unwrap().end(counter_index.unwrap());
    }

}

fn try_ptr_parallel() {
    let size = 100_000;
    let thread_min_fraction = 0.0;
    let thread_min_size = 25_0000;
    let mut v = vec_usize_shuffled(size);
    // dbg!(&v);
    let start = Instant::now();
    let counter = quicksort_parallel_ptr(&mut v[..], thread_min_fraction, thread_min_size, false, 7);
    dbg!(start.elapsed());
    // dbg!(&v[..10], &v[v.len() - 10..]);
    dbg!(&counter.is_some());
    if let Some(ct) = counter {
        //dbg!(&ct);
        // ct.dbg(Some(1));
        ct.dbg(None);
    }
    // assert_eq!(size, v.len());
    // assert!(&v.is_sorted());

}
