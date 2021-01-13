#![allow(dead_code)]

use rayon::prelude::*;
use std::fmt::{self, Debug};
use std::time;
use std::marker::Sync;
use std::ops::{Deref, DerefMut};

use crate::util;
use crate::sort::test_data;
use crate::sort::bubble_sort;
use std::borrow::Borrow;

pub fn main() {
    // try_sort_small();
    try_log();
}

#[derive(Debug, Copy, Clone)]
pub enum Actor {
    Quicksort,
    BubbleSort,
}

#[derive(Debug)]
pub struct Swap {
    instant: time::Instant,
    a: usize,
    b: usize,
}

#[derive(Debug)]
pub struct RangeCall {
    key: String,
    actor: Actor,
    from: usize,
    len: usize,
    is_new_thread: bool,
    start_instant: time::Instant,
    end_instant: Option<time::Instant>,
    swaps: Vec<Swap>,
    child_calls: Vec<RangeCall>,
    thread_number: u8,
}

#[derive(Clone)]
pub enum ActionType {
    MoveToThread,
    Take,
    Release,
    Swap {
        a: usize,
        b: usize,
    },
}

#[derive(Clone)]
pub struct Action {
    duration: time::Duration,
    thread_number: u8,
    call_key: String,
    actor: Actor,
    from: usize,
    to: usize,
    action_type: ActionType,
}

#[derive(Clone)]
struct ActionList(Vec<Action>);

impl RangeCall {
    fn new(key: &str, actor: Actor, from: usize, len: usize, is_new_thread: bool) -> Self {
        Self {
            key: key.to_string(),
            actor,
            from,
            len,
            is_new_thread,
            start_instant: time::Instant::now(),
            end_instant: None,
            swaps: vec![],
            child_calls: vec![],
            thread_number: 0,
        }
    }
    
    fn end(&mut self) {
        self.end_instant = Some(time::Instant::now());
    }

    fn add_child_call(&mut self, child_call: RangeCall) {
        self.child_calls.push(child_call);
    }

    fn add_child_calls(&mut self, calls: (RangeCall, RangeCall)) {
        self.child_calls.push(calls.0);
        self.child_calls.push(calls.1);
    }

    fn swap(&mut self, a: usize, b: usize) {
        // These are relative offsets. We'll work out the absolute offsets later.
        debug_assert!(a != b, "a == b == {}", a);
        debug_assert!(a < self.len, "len = {}; a = {}", self.len, a);
        debug_assert!(b < self.len, "len = {}; b = {}", self.len, b);
        self.swaps.push(Swap { instant: time::Instant::now(), a, b });
    }

    fn to(&self) -> usize {
        self.from + self.len
    }
}

impl ActionType {

    pub fn description_width(&self, range_width: usize) -> String {
        match self {
            ActionType::MoveToThread                 => "MoveToThread".to_string(),
            ActionType::Take                         => "Take        ".to_string(),
            ActionType::Release                      => "Release     ".to_string(),
            ActionType::Swap { a, b } => format!("Swap         ({:>width$}, {:>width$})", util::format::format_count(*a), util::format::format_count(*b), width = range_width),
        }
    }

    pub fn description(&self) -> String {
        self.description_width(0)
    }
}

impl fmt::Debug for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Action {

    fn new(duration: time::Duration, thread_number: u8, range_call: &RangeCall, action_type: ActionType) -> Self {
        Self {
            duration,
            thread_number,
            call_key: range_call.key.clone(),
            actor: range_call.actor,
            from: range_call.from,
            to: range_call.to(),
            action_type
        }
    }

    /*
    pub fn new_move_to_thread(duration: time::Duration, thread_number: u8, range_call: &RangeCall) -> Self {
        Self { duration, thread_number, from, to, action_type: ActionType::MoveToThread }
    }

    pub fn new_take(duration: time::Duration, thread_number: u8, range_call: &RangeCall, actor: Actor) -> Self {
        Self { duration, thread_number, from, to, action_type: ActionType::Take { actor } }
    }

    pub fn new_release(duration: time::Duration, thread_number: u8, range_call: &RangeCall) -> Self {
        Self { duration, thread_number, from, to, action_type: ActionType::Release }
    }

    pub fn new_swap(duration: time::Duration, thread_number: u8, range_call: &RangeCall, a: usize, b: usize) -> Self {
        Self { duration, thread_number, from, to, action_type: ActionType::Swap { a, b } }
    }
    */

    fn add_actions_from_range_call(action_list: &mut ActionList, range_call: &RangeCall, first_instant: time::Instant, thread_starters: &Vec<(String, time::Instant)>, mut thread_number: u8) {

        let start_duration = range_call.start_instant.duration_since(first_instant);

        if range_call.is_new_thread {
            let range_call_key = range_call.key.clone();
            thread_number = Self::key_to_thread_number(thread_starters, range_call_key);
            let thread_duration = just_before(&start_duration);
            action_list.push(Action::new(thread_duration, thread_number, range_call, ActionType::MoveToThread));
        }

        action_list.push(Action::new(start_duration, thread_number, range_call,ActionType::Take));

        for swap in range_call.swaps.iter() {
            let swap_duration = swap.instant.duration_since(first_instant);
            let action_type = ActionType::Swap { a: range_call.from + swap.a, b: range_call.from + swap.b };
            action_list.push(Action::new(swap_duration, thread_number, range_call, action_type));
        }

        // If there are child calls, given the min start time and the max end time of the calls, have the current
        // parent call insert a Release action just before the min start time and a Take just after the max end time.
        // This will cover several cases, such as when one child thread finishes before the other, or when running
        // synchronously the first half of the slice should show work while the second half is idle.

        if range_call.child_calls.len() > 0 {

                // The child calls happened on new threads so we need to give away the child ranges explicitly in case
                // there's a delay before the child calls start, or a delay after

            let earliest_start_instant = range_call.child_calls.iter().map(|x| x.start_instant).min().unwrap();
            let before_earliest_start_duration = just_before(&earliest_start_instant.duration_since(first_instant));

            let latest_end_instant = range_call.child_calls.iter().map(|x| x.end_instant.unwrap()).max().unwrap();
            let after_latest_end_duration = just_after(&latest_end_instant.duration_since(first_instant));

            for (index, child_call) in range_call.child_calls.iter().enumerate() {

                if index == 1 || child_call.is_new_thread {
                    // Either this is the second child call in the same thread or it's a child call in a new
                    // thread, or both. Give away ownership of the child range just before the first of the child
                    // calls begins, in case there's a delay before the child call starts.
                    action_list.push(Action::new(before_earliest_start_duration, thread_number, child_call, ActionType::Release));
                }

                // Add the Action items for the child call.
                Self::add_actions_from_range_call(action_list, child_call, first_instant.clone(), thread_starters, thread_number);

                if child_call.is_new_thread {
                    // We consider the range to be returned to the current call just after each child call ends even if
                    // we're waiting for the other child call. In the final collection of Action items this will be
                    // symmetrical, with a matching Action that moves the range into the child thread. But at the level
                    // of the current call we don't know the thread number for the child call, so the corresponding
                    // Action was added above in the from_range_call() call for the child call.
                    let thread_duration = just_after(&child_call.end_instant.unwrap().duration_since(first_instant));
                    action_list.push(Action::new(thread_duration, thread_number, child_call, ActionType::MoveToThread));
                }

                if index == 0 || child_call.is_new_thread {
                    // Either this was the first child call in the same thread or it's a child call in a new thread, or
                    // both. Take back ownership of the current range just after the later of the child calls ends.
                    let release_duration = child_call.end_instant.unwrap().duration_since(first_instant);
                    action_list.push(Action::new(release_duration, thread_number, child_call, ActionType::Release));
                    action_list.push(Action::new(after_latest_end_duration, thread_number, child_call, ActionType::Take));
                }
            }
        }

    }

    fn key_to_thread_number(thread_starters: &Vec<(String, time::Instant)>, range_call_key: String) -> u8 {
        for i in 0..thread_starters.len() {
            if thread_starters[i].0 == range_call_key {
                return i as u8;
            }
        };
        panic!("Matching item not found.")
    }


    pub fn description_width(&self, duration_width: usize, thread_number_width: usize, actor_width: usize, range_width: usize) -> String {
        let duration = format!("time = {:>width$}", describe_duration_nanos(&self.duration), width = duration_width);
        let thread = format!("thread = {:>width$}", self.thread_number, width = thread_number_width);
        let actor = format!( "{:<width$}", format!("{:?}", self.actor), width = actor_width);
        let range = format!("[{:>width$}..{:>width$}]", util::format::format_count(self.from), util::format::format_count(self.to), width = range_width);
        let action_type = self.action_type.description_width(range_width);
        format!("{}; {}; {}; {}; {}", duration, thread, actor, range, action_type)
    }

    pub fn description(&self) -> String {
        self.description_width(0, 0, 0, 0)
    }

    pub fn is_detail(&self) -> bool {
        match self.action_type {
            ActionType::MoveToThread | ActionType::Take {..} | ActionType::Release => false,
            _ => true,
        }
    }

    pub fn call_key_as_int(&self) -> u32 {
        u32::from_str_radix(&self.call_key, 2).unwrap()
    }
}

impl ActionList {

    pub fn from_range_call(range_call: &RangeCall) -> Self {
        let mut thread_starters: Vec<(String, time::Instant)> = vec![];
        Self::get_thread_starters(&mut thread_starters, &range_call);
        thread_starters.sort_unstable_by_key(|x| x.1);

        let mut action_list = ActionList { 0: vec![] };
        Action::add_actions_from_range_call(&mut action_list, range_call, range_call.start_instant.clone(), &thread_starters, 0);
        action_list
    }

    fn get_thread_starters(v: &mut Vec<(String, time::Instant)>, range_call: &RangeCall) {
        if range_call.is_new_thread {
            v.push((range_call.key.clone(), range_call.start_instant));
        }
        for child_call in range_call.child_calls.iter() {
            Self::get_thread_starters(v, child_call);
        }
    }

    pub fn clone_details_optional(&self, include_detail_actions: bool) -> Self {
        ActionList {
            0: self.iter()
                .filter(|x| include_detail_actions || !x.is_detail())
                .map(|x| x.clone())
                .collect()
        }
    }

    pub fn display_by_time(&self, include_detail_actions: bool) {
        self.display_internal("Actions by Time", include_detail_actions, |x| x.duration, |_x| 1);
    }

    pub fn display_by_thread(&self, include_detail_actions: bool) {
        self.display_internal("Actions by Thread", include_detail_actions, |x|  (x.thread_number, x.duration), |_x| 1);
    }

    pub fn display_by_range(&self, include_detail_actions: bool) {
        self.display_internal("Actions by Range", include_detail_actions, |x| (x.from, 0 - x.to as isize, x.duration), |_x| 1);
    }

    pub fn display_by_call_key(&self, include_detail_actions: bool) {
        self.display_internal("Actions by Call Key", include_detail_actions, |x| ((x.call_key_as_int(), x.call_key.len(), x.duration)), |x| x.call_key.len());
    }

    pub fn display_internal<K, F, D>(&self, label: &str, include_detail_actions: bool, key_func: F, depth_func: D)
        where
            F: FnMut(&Action) -> K,
            K: Ord,
            D: Fn(&Action) -> usize,
    {
        let details_label = if include_detail_actions { "" } else { " (details omitted)" };
        println!("\n{}{}:", label, details_label);
        let mut action_list = self.clone_details_optional(include_detail_actions);
        action_list.sort_unstable_by_key(key_func);

        let duration_width = action_list.iter().map(|x| describe_duration_nanos(&x.duration).len()).max().unwrap();
        let thread_number_width = action_list.iter().map(|x| util::format::format_count(x.thread_number).len()).max().unwrap();
        let actor_width = action_list.iter().map(|x| format!("{:?}", x.actor).len()).max().unwrap();
        let range_width = action_list.iter().map(|x| util::format::format_count(x.to).len()).max().unwrap();

        for action in action_list.iter() {
            let depth = depth_func(action);
            util::print_indent(depth, &action.description_width(duration_width, thread_number_width, actor_width, range_width));
        }
    }

}

impl Deref for ActionList {
    type Target = Vec<Action>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActionList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

fn describe_duration_nanos(duration: &time::Duration) -> String {
    util::format::format_count(duration.as_nanos())
}

fn just_before(duration: &time::Duration) -> time::Duration {
    *duration - time::Duration::from_micros(1)
}

fn just_after(duration: &time::Duration) -> time::Duration {
    *duration + time::Duration::from_micros(1)
}

pub fn quicksort_rayon_vis<T>(s: &mut [T], min_split_size: usize, max_threads: u8) -> RangeCall
    where T: Ord + Send + Debug
{
    quicksort_rayon_vis_internal(s, 0, false, min_split_size, max_threads, "0".to_string())
}
    
pub fn quicksort_rayon_vis_internal<T>(s: &mut [T], from: usize, is_new_thread: bool, min_split_size: usize, max_threads: u8, range_call_key: String) -> RangeCall
    where T: Ord + Send + Debug
{
    let s_len = s.len();
    let mut range_call = RangeCall::new(&range_call_key, Actor::Quicksort, from, s_len, is_new_thread);
    if s_len > 1 {
        if s_len < min_split_size {
            range_call.child_calls.push(bubble_sort_vis(s, from, format!("{}0", &range_call_key)));
        } else {
            let mid = partition(s);
            let (lo, hi) = s.split_at_mut(mid);
            let key_lo = format!("{}0", &range_call_key);
            let key_hi = format!("{}1", &range_call_key);
            if max_threads == 1 {
                range_call.add_child_call(quicksort_rayon_vis_internal(lo, from, false, min_split_size, max_threads, key_lo));
                range_call.add_child_call(quicksort_rayon_vis_internal(hi, from + mid, false, min_split_size, max_threads, key_hi));
            } else {
                range_call.add_child_calls(rayon::join(
                    || quicksort_rayon_vis_internal(lo, from, true, min_split_size, max_threads / 2, key_lo),
                    || quicksort_rayon_vis_internal(hi, from + mid, true, min_split_size, max_threads / 2, key_hi)
                ));
            }
        }
    }
    range_call.end();
    range_call
}

#[inline]
fn partition<T> (s: &mut [T]) -> usize
    where T: Ord + Send + Debug
{
    let pivot = if s.len() <= 3 {
        1
    } else {
        // Median of the first, middle, and last elements.
        let mut pivots = [0, s.len() / 2, s.len() - 1];
        pivots.sort_unstable_by_key(|i| &s[*i]);
        pivots[1]
    };
    s.partition_at_index(pivot);
    pivot
}

#[inline]
pub fn bubble_sort_vis<T> (s: &mut [T], from: usize, range_call_key: String) -> RangeCall
    where T: PartialOrd + Debug
{
    let s_len = s.len();
    let mut range_call = RangeCall::new(&range_call_key, Actor::BubbleSort, from, s_len, false);
    let mut did_swap = true;
    for i in (0..s_len).rev() {
        if !did_swap {
            range_call.end();
            return range_call;
        }
        did_swap = false;
        for j in 0..i {
            if s[j] > s[j + 1] {
                s.swap(j, j + 1);
                range_call.swap(j, j + 1);
                did_swap = true;
            }
        }
    }
    range_call.end();
    range_call
}

fn try_sort_small() {
    let min_split_size = 10;
    let max_threads = 4;
    for size in 1..= 100 {
        let mut v = test_data::vec_usize_shuffled(size);
        // bg!(&v);
        // quicksort_rayon_minimal(&mut v);
        quicksort_rayon_vis(&mut v, min_split_size, max_threads);
        dbg!(&v);
        assert!(v.is_sorted());
    }
}

fn try_log() {
    let min_split_size = 10;
    let max_threads = 1;
    let size = 8;
    let mut v = test_data::vec_usize_shuffled(size);
    dbg!(&v);
    let range_call = quicksort_rayon_vis(&mut v, min_split_size, max_threads);
    assert!(v.is_sorted());
    let action_list = ActionList::from_range_call(&range_call);
    action_list.display_by_time(true);
    action_list.display_by_thread(true);
    action_list.display_by_range(true);
    action_list.display_by_call_key(true);
}
