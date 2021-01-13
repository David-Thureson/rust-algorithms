use std::fmt::Debug;

use super::model::*;
use crate::sort::test_data;

pub fn main() {
    try_sort_small();
    try_log();
}

#[inline]
pub fn bubble_sort<T> (s: &mut [T], from: usize, range_call_key: &str) -> RangeCall
    where T: PartialOrd + Debug
{
    //bg!(&s);
    let s_len = s.len();
    let mut range_call = RangeCall::new(range_call_key, Actor::BubbleSort, from, s_len, false);
    for i in (1..s_len).rev() {
        let mut did_swap = false;
        for j in 0..i {
            if s[j] > s[j + 1] {
                s.swap(j, j + 1);
                range_call.swap(j, j + 1);
                did_swap = true;
            }
        }
        if did_swap {
            //rintln!("a. Marking final = {}..{}", i, i + 1);
            range_call.mark_final_one(i);
        } else {
            //rintln!("b. Marking final = {}..{}", 0, i);
            range_call.mark_final(0, i + 1);
            range_call.end();
            debug_assert!(range_call.debug_check_mark_finals_coverage(false));
            return range_call;
        }
    }
    //rintln!("c. Marking final = {}..{}", 0, 1);
    range_call.mark_final_one(0);
    range_call.end();
    debug_assert!(range_call.debug_check_mark_finals_coverage(false));
    range_call
}

fn try_sort_small() {
    for size in 1..= 100 {
        let mut v = test_data::vec_usize_shuffled(size);
        // bg!(&v);
        // quicksort_rayon_minimal(&mut v);
        bubble_sort(&mut v, 0, "0");
        dbg!(&v);
        assert!(v.is_sorted());
    }
}

fn try_log() {
    let size = 12;
    let include_detail_actions = true;
    let mut v = test_data::vec_usize_shuffled(size);
    bubble_sort(&mut v, 0, "0").try_log(include_detail_actions);
}
