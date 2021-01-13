pub mod bubble_sort;
pub use bubble_sort::*;

pub mod insertion_sort;
pub use insertion_sort::*;

pub mod merge_sort;
pub use merge_sort::*;
// pub use merge_sort::{merge, merge_sort, merge_sort_with_bubble, merge_sort_test_only_no_merge};

pub mod merge_sort_loop;

pub mod quicksort_crossbeam;

pub mod quicksort_ptr;
pub use quicksort_ptr::*;

pub mod quicksort_rayon;

pub mod quicksort_safe;

// pub mod sort_vis;

pub mod test_data;

use std::fmt::{self, Debug};

use crate::*;
use counter::{Counter, CounterItem};

// pub trait <T: PartialOrd + Debug> Sortable<T>;
// pub trait Sortable: PartialOrd + Debug {}

#[derive(Clone)]
pub struct SliceCounterItem {
    pub start_index: usize,
    pub end_index: usize,
    pub method: Option<String>,
}

impl Debug for SliceCounterItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = self.method.as_ref().map_or("".to_string(), |x| format!("; method = {}", x));
        write!(f, "[{}..{}){}", self.start_index, self.end_index, method)
    }
}

impl Counter<SliceCounterItem> {
    pub fn dbg(&self, max_depth: Option<usize>) {
        println!("{}", self.describe_deep(max_depth, &|a: &(usize, &CounterItem<SliceCounterItem>), b: &(usize, &CounterItem<SliceCounterItem>)|
            a.1.data.as_ref().unwrap().start_index.cmp(&b.1.data.as_ref().unwrap().start_index)
        ));
    }
}
