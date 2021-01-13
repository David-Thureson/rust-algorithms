#![allow(dead_code)]

use std::fmt::{self, Debug};
use std::time;
use std::marker::Sync;
use std::ops::{Deref, DerefMut, Range};

use crate::util::format;
use crate::range;
use std::collections::HashMap;
use itertools::Itertools;
// use std::iter::Filter;
// use std::slice::Iter;
// use itertools::Itertools;

pub fn main() {
}

#[derive(Debug, Copy, Clone)]
pub enum Actor {
    Quicksort,
    BubbleSort,
    Partition,
}

#[derive(Debug)]
pub struct Swap {
    instant: time::Instant,
    a: usize,
    b: usize,
}

#[derive(Debug)]
pub struct MarkFinal {
    instant: time::Instant,
    from: usize,
    to: usize,
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
    mark_finals: Vec<MarkFinal>,
    child_calls: Vec<RangeCall>,
    thread_number: u8,
}

#[derive(Clone)]
pub enum ActionType {
    MoveToThread,
    MoveToCall {
        from_call_key: String,
        to_call_key: String,
        ranges: Vec<Range<usize>>,
    },
    Take,
    Release,
    Swap {
        a: usize,
        b: usize,
    },
    MarkFinal {
        from: usize,
        to: usize,
    },
}

#[derive(Clone)]
pub struct Action {
    pub duration: time::Duration,
    pub thread_number: u8,
    // pub run_number: Option<u8>,
    pub call_key: String,
    pub actor: Actor,
    pub from: usize,
    pub to: usize,
    pub action_type: ActionType,
}

#[derive(Clone)]
pub struct ActionList {
    pub label: Option<String>,
    pub actions: Vec<Action>,
    pub unsorted_array: Option<Vec<usize>>,
}

impl RangeCall {
    pub fn new(key: &str, actor: Actor, from: usize, len: usize, is_new_thread: bool) -> Self {
        Self {
            key: key.to_string(),
            actor,
            from,
            len,
            is_new_thread,
            start_instant: time::Instant::now(),
            end_instant: None,
            swaps: vec![],
            mark_finals: vec![],
            child_calls: vec![],
            thread_number: 0,
        }
    }

    pub fn end(&mut self) {
        self.end_instant = Some(time::Instant::now());
    }

    pub fn add_child_call(&mut self, child_call: RangeCall) {
        self.child_calls.push(child_call);
    }

    pub fn add_child_calls(&mut self, calls: (RangeCall, RangeCall)) {
        self.child_calls.push(calls.0);
        self.child_calls.push(calls.1);
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        // These are relative offsets. We'll work out the absolute offsets later.
        debug_assert!(a != b, "a == b == {}", a);
        debug_assert!(a < self.len, "len = {}; a = {}", self.len, a);
        debug_assert!(b < self.len, "len = {}; b = {}", self.len, b);
        self.swaps.push(Swap { instant: time::Instant::now(), a, b });
    }

    pub fn mark_final_one(&mut self, index: usize) {
        self.mark_final(index, index + 1);
    }

    pub fn mark_final(&mut self, from: usize, to: usize) {
        debug_assert!(from < to);
        debug_assert!(from < self.len);
        debug_assert!(to <= self.len);
        self.mark_finals.push(MarkFinal { instant: time::Instant::now(), from, to });
    }

    fn to(&self) -> usize {
        self.from + self.len
    }

    pub fn try_log(&self, include_detail_actions: bool) {
        let action_list = ActionList::from_range_call(&self, None);
        action_list.display_by_time(include_detail_actions);
        action_list.display_by_thread(include_detail_actions);
        action_list.display_by_range(include_detail_actions);
        action_list.display_by_call_key(include_detail_actions);
    }

    pub fn is_same_range(&self, other: &RangeCall) -> bool {
        other.from == self.from && other.to() == self.to()
    }

    pub fn is_subrange(&self, other: &RangeCall) -> bool {
        other.from >= self.from && other.to() <= self.to() && other.len < self.len
    }

    /*
    pub fn child_calls_same_range<P>(&self) -> Filter<Iter<RangeCall>, P>
        where P: FnMut(&RangeCall) -> bool
    {
        // let mut iter: Filter<Iter<&RangeCall>, P> = self.child_calls.iter().as_ref().filter(|x| self.is_same_range(&x));
        let mut iter = self.child_calls.iter().filter(|_x| true);
        iter
    }

    pub fn child_calls_subrange<P>(&self) -> Filter<RangeCall, P>
        where P: FnMut(&RangeCall) -> bool
    {
        self.child_calls.iter().filter(|x| self.is_subrange(x))
    }
    */

    pub fn child_calls_same_thread(&self) -> Vec<&RangeCall> {
        self.child_calls.iter().filter(|x| !x.is_new_thread).collect()
    }

    pub fn child_calls_different_threads(&self) -> Vec<&RangeCall> {
        self.child_calls.iter().filter(|x| x.is_new_thread).collect()
    }

    pub fn child_calls_same_range(&self) -> Vec<&RangeCall> {
        self.child_calls.iter().filter(|x| self.is_same_range(x)).collect()
    }

    pub fn child_calls_subrange(&self) -> Vec<&RangeCall> {
        self.child_calls.iter().filter(|x| self.is_subrange(x)).collect()
    }

    pub fn debug_check_mark_finals_coverage(&self, always_display: bool) -> bool {
        let complete = self.are_mark_finals_complete();
        if always_display || !complete {
            if complete {
                println!("\nMark finals cover the range:");
            } else {
                println!("\nMark finals don't cover the range:");
            }
            println!("len = {}", self.len);
            for mark_final in self.mark_finals.iter().sorted_by_key(|x| x.from) {
                println!("\t{}..{}", mark_final.from, mark_final.to);
            }
        }
        complete
    }

    pub fn are_mark_finals_complete(&self) -> bool {
        let mut prev_to = None;
        for (index, mark_final) in self.mark_finals.iter().sorted_by_key(|x| x.from).enumerate() {
            //bg!(index, &prev_to);
            if index == 0 {
                if mark_final.from != 0 {
                    return false;
                }
            } else {
                if mark_final.from != prev_to.unwrap() {
                    return false;
                }
            }
            prev_to = Some(mark_final.to);
        }
        if prev_to.unwrap() != self.len {
            return false;
        }
        return true;
    }
}

impl ActionType {

    pub fn move_to_thread() -> Self { ActionType::MoveToThread }

    pub fn take() -> Self { ActionType::Take }

    pub fn release() -> Self { ActionType::Release }

    pub fn swap(a: usize, b: usize) -> Self {
        debug_assert!(a != b);
        ActionType::Swap { a, b }
    }

    pub fn mark_final(from: usize, to: usize) -> Self {
        debug_assert!(from < to);
        ActionType::MarkFinal { from, to }
    }

    pub fn description_width(&self, range_width: usize) -> String {
        match self {
            ActionType::MoveToThread => "MoveToThread".to_string(),
            ActionType::MoveToCall { from_call_key, to_call_key, ranges: _} => format!("MoveToCall from \"{}\" to \"{}\"", from_call_key, to_call_key),
            ActionType::Take => "Take        ".to_string(),
            ActionType::Release => "Release     ".to_string(),
            ActionType::Swap { a, b } => format!("Swap         ({:>width$}, {:>width$})", util::format::format_count(*a), util::format::format_count(*b), width = range_width),
            ActionType::MarkFinal { from, to } => format!("MarkFinal   {}({:>width$}..{:>width$})", " ".repeat((2 * range_width) + 6), util::format::format_count(*from), util::format::format_count(*to), width = range_width),
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
        Self::new_detail(duration, thread_number, &range_call.key, range_call.actor, range_call.from, range_call.to(), action_type)
    }

    fn new_detail(duration: time::Duration, thread_number: u8, call_key: &str, actor: Actor, from: usize, to: usize, action_type: ActionType) -> Self {
        debug_assert!(
            match action_type {
                ActionType::Swap { a, b } => a >= from && a < to && b >= from && b < to,
                ActionType::MarkFinal { from: final_from, to: final_to } => final_from >= from && final_from < to && final_to > from && final_to <= to,
                _ => true,
            }
        );
        Self {
            duration,
            thread_number,
            call_key: call_key.to_string(),
            actor,
            from,
            to,
            action_type
        }
    }

    fn add_actions_from_range_call(action_list: &mut ActionList, range_call: &RangeCall, first_instant: time::Instant, thread_starters: &Vec<(String, time::Instant)>, mut thread_number: u8) {

        let start_duration = range_call.start_instant.duration_since(first_instant);
        let end_duration = range_call.end_instant.unwrap().duration_since(first_instant);

        let parent_thread_number = thread_number;

        if range_call.is_new_thread {
            let range_call_key = range_call.key.clone();
            thread_number = Self::key_to_thread_number(thread_starters, range_call_key);
            let thread_duration = just_before(&start_duration);
            action_list.push(Action::new(thread_duration, thread_number, range_call, ActionType::move_to_thread()));
        }

        action_list.push(Action::new(start_duration, thread_number, range_call,ActionType::take()));

        for swap in range_call.swaps.iter() {
            let swap_duration = swap.instant.duration_since(first_instant);
            let action_type = ActionType::swap(range_call.from + swap.a, range_call.from + swap.b);
            action_list.push(Action::new(swap_duration, thread_number, range_call, action_type));
        }

        for mark_final in range_call.mark_finals.iter() {
            let mark_final_duration = mark_final.instant.duration_since(first_instant);
            let action_type = ActionType::mark_final(range_call.from + mark_final.from, range_call.from + mark_final.to);
            action_list.push(Action::new(mark_final_duration, thread_number, range_call, action_type));
        }

        if range_call.child_calls.len() > 0 {
            let earliest_start_instant = range_call.child_calls.iter().map(|x| x.start_instant).min().unwrap();
            let before_earliest_start_duration = just_before(&just_before(&earliest_start_instant.duration_since(first_instant)));

            let latest_end_instant = range_call.child_calls.iter().map(|x| x.end_instant.unwrap()).max().unwrap();
            let after_latest_end_duration = just_after(&just_after(&latest_end_instant.duration_since(first_instant)));

            action_list.push(Action::new(before_earliest_start_duration, thread_number, range_call, ActionType::Release));

            for child_call in range_call.child_calls.iter() {

                let before_call_duration = just_before(&child_call.start_instant.duration_since(first_instant));
                let move_to_call_action_type = ActionType::MoveToCall { from_call_key: range_call.key.clone(), to_call_key: child_call.key.clone(), ranges: vec![] };
                action_list.push(Action::new_detail(before_call_duration, thread_number, &range_call.key, range_call.actor, child_call.from, child_call.to(), move_to_call_action_type));

                Action::add_actions_from_range_call(action_list, child_call, first_instant.clone(), thread_starters, thread_number);

                let after_call_duration = just_after(&child_call.end_instant.unwrap().duration_since(first_instant));
                let move_to_call_action_type = ActionType::MoveToCall { from_call_key: child_call.key.clone(), to_call_key: range_call.key.clone(), ranges: vec![] };
                action_list.push(Action::new_detail(after_call_duration, thread_number, &range_call.key, range_call.actor, child_call.from, child_call.to(), move_to_call_action_type));

            }

            action_list.push(Action::new(after_latest_end_duration, thread_number, range_call, ActionType::Take));
        }

        action_list.push(Action::new(end_duration, thread_number, range_call, ActionType::Release));

        if range_call.is_new_thread {
            // Return this range to the parent thread.
            let thread_duration = just_after(&end_duration);
            action_list.push(Action::new(thread_duration, parent_thread_number, range_call, ActionType::move_to_thread()));
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

    pub fn from_range_call(range_call: &RangeCall, label: Option<&str>) -> Self {
        let mut thread_starters: Vec<(String, time::Instant)> = vec![];
        Self::get_thread_starters(&mut thread_starters, &range_call);
        thread_starters.sort_unstable_by_key(|x| x.1);

        let mut action_list = ActionList {
            label: label.map(|label| label.to_string()),
            actions: vec![],
            unsorted_array: None
        };
        Action::add_actions_from_range_call(&mut action_list, range_call, range_call.start_instant.clone(), &thread_starters, 0);
        action_list.resolve_call_ranges(range_call.len);
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
            label: self.label.clone(),
            actions: self.iter()
                .filter(|x| include_detail_actions || !x.is_detail())
                .map(|x| x.clone())
                .collect(),
            unsorted_array: self.unsorted_array.clone(),
        }
    }

    fn resolve_call_ranges(&mut self, full_range_end: usize) {
        let mut map = HashMap::new();

        let mut first_call = range::RangeSet::new(true, true, Some(0..full_range_end));
        first_call.add_range(&(0..full_range_end));
        debug_assert!(first_call.check_ranges(false, Some("first_call"), None));
        map.insert("0".to_string(), first_call);

        for action in self.iter_mut().sorted_by_key(|action| action.duration) {
            match &mut action.action_type {
                ActionType::MoveToCall { from_call_key, to_call_key, ranges: action_ranges } => {
                    {
                        let from_call = map.get_mut(from_call_key).unwrap();
                        from_call.subtract_range(&(action.from..action.to));
                        debug_assert!(from_call.check_ranges(false, Some("from_call"), None));
                    }

                    if let Some(to_call) = map.get_mut(to_call_key) {
                        to_call.add_range(&(action.from..action.to));
                        debug_assert!(to_call.check_ranges(false, Some("to_call"), None));
                    } else {
                        let mut new_call = range::RangeSet::new(true, true, Some(action.from..action.to));
                        new_call.add_range(&(action.from..action.to));
                        debug_assert!(new_call.check_ranges(false, Some("new_call"), None));
                        map.insert(to_call_key.clone(), new_call);
                    }

                    let mut combined_ranges = range::RangeSet::new(true, false, Some(0..full_range_end));
                    for range in map.values().map(|range_set| range_set.ranges_in_order()).flatten() {
                        combined_ranges.add_range(&(range.start..range.end));
                    }
                    combined_ranges.allow_gaps = false;
                    debug_assert!(combined_ranges.check_ranges(false, Some("combined_ranges"), None));
                    for range in combined_ranges.ranges_in_order() {
                        action_ranges.push(range.start..range.end);
                    }
                },
                _ => (),
            }
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
        // self.display_internal("Actions by Call Key", include_detail_actions, |x| (x.call_key_as_int(), x.call_key.len(), x.duration), |x| x.call_key.len());
        self.display_internal("Actions by Call Key", include_detail_actions, |x| x.duration, |x| x.call_key.len());
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
            format::println_indent_space(depth, &action.description_width(duration_width, thread_number_width, actor_width, range_width));
        }
    }

    pub fn report_types(&self) {
        let mut move_to_thread = 0;
        let mut move_to_call = 0;
        let mut take = 0;
        let mut release = 0;
        let mut swap = 0;
        let mut mark_final = 0;
        for action in self.actions.iter() {
            match action.action_type {
                ActionType::MoveToThread => { move_to_thread += 1; },
                ActionType::MoveToCall { from_call_key: _, to_call_key: _, ranges: _ } => { move_to_call += 1; },
                ActionType::Take => { take += 1; },
                ActionType::Release => { release += 1; },
                ActionType::Swap { a: _, b: _ } => { swap += 1; },
                ActionType::MarkFinal { from: _ , to: _ } => { mark_final += 1; },
            }
        }
        println!("\nmove to thread = {}", format::format_count(move_to_thread));
        println!("move to call = {}", format::format_count(move_to_call));
        println!("take = {}", format::format_count(take));
        println!("release = {}", format::format_count(release));
        println!("swap = {}", format::format_count(swap));
        println!("mark_final = {}\n", format::format_count(mark_final));
    }

}

impl Deref for ActionList {
    type Target = Vec<Action>;

    fn deref(&self) -> &Self::Target {
        &self.actions
    }
}

impl DerefMut for ActionList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.actions
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
    *duration - time::Duration::from_nanos(1)
}

fn just_after(duration: &time::Duration) -> time::Duration {
    *duration + time::Duration::from_nanos(1)
}

/*
    fn add_actions_from_range_call(action_list: &mut ActionList, range_call: &RangeCall, first_instant: time::Instant, thread_starters: &Vec<(String, time::Instant)>, mut thread_number: u8) {

        let start_duration = range_call.start_instant.duration_since(first_instant);
        let end_duration = range_call.end_instant.unwrap().duration_since(first_instant);

        if range_call.is_new_thread {
            let range_call_key = range_call.key.clone();
            thread_number = Self::key_to_thread_number(thread_starters, range_call_key);
            let thread_duration = just_before(&start_duration);
            action_list.push(Action::new(thread_duration, thread_number, range_call, ActionType::move_to_thread()));
        }

        action_list.push(Action::new(start_duration, thread_number, range_call,ActionType::take()));

        for swap in range_call.swaps.iter() {
            let swap_duration = swap.instant.duration_since(first_instant);
            let action_type = ActionType::swap(range_call.from + swap.a, range_call.from + swap.b);
            action_list.push(Action::new(swap_duration, thread_number, range_call, action_type));
        }

        for mark_final in range_call.mark_finals.iter() {
            let mark_final_duration = mark_final.instant.duration_since(first_instant);
            let action_type = ActionType::mark_final(range_call.from + mark_final.from, range_call.from + mark_final.to);
            action_list.push(Action::new(mark_final_duration, thread_number, range_call, action_type));
        }

        for child_call in range_call.child_calls_same_thread() {

            let release_duration = just_before(&child_call.start_instant.duration_since(first_instant));
            action_list.push(Action::new(release_duration, thread_number, range_call,ActionType::release()));

            // Add the Action items for the child call.
            Self::add_actions_from_range_call(action_list, child_call, first_instant.clone(), thread_starters, thread_number);

            let take_duration = just_after(&child_call.end_instant.unwrap().duration_since(first_instant));
            action_list.push(Action::new(take_duration, thread_number, range_call,ActionType::take()));
        }

        // If there are child calls, given the min start time and the max end time of the calls, have the current
        // parent call insert a Release action just before the min start time and a Take just after the max end time.
        // This will cover several cases, such as when one child thread finishes before the other, or when running
        // synchronously the first half of the slice should show work while the second half is idle.


        if range_call.child_calls_subrange().len() > 0 {

            // These are the paired child calls with subranges which may run in their own threads, as opposed to single
            // calls such as to bubble_sort() or partition() that take the whole range and run in the same thread.

            let earliest_start_instant = range_call.child_calls_subrange().iter().map(|x| x.start_instant).min().unwrap();
            let before_earliest_start_duration = just_before(&earliest_start_instant.duration_since(first_instant));

            let latest_end_instant = range_call.child_calls_subrange().iter().map(|x| x.end_instant.unwrap()).max().unwrap();
            let after_latest_end_duration = just_after(&latest_end_instant.duration_since(first_instant));

            action_list.push(Action::new(before_earliest_start_duration, thread_number, range_call, ActionType::Release));

            for (_index, child_call) in range_call.child_calls_subrange().iter().enumerate() {

                // if index == 1 || child_call.is_new_thread {
                    // Either this is the second child call in the same thread or it's a child call in a new
                    // thread, or both. Give away ownership of the child range just before the first of the child
                    // calls begins, in case there's a delay before the child call starts.
                    // action_list.push(Action::new_detail(before_earliest_start_duration, thread_number, &range_call.key, range_call.actor, child_call.from, child_call.to(),ActionType::release()));
                // }

                // Add the Action items for the child call.
                Self::add_actions_from_range_call(action_list, child_call, first_instant.clone(), thread_starters, thread_number);

                if child_call.is_new_thread {
                    // We consider the range to be returned to the current call just after each child call ends even if
                    // we're waiting for the other child call. In the final collection of Action items this will be
                    // symmetrical, with a matching Action that moves the range into the child thread. But at the level
                    // of the current call we don't know the thread number for the child call, so the corresponding
                    // Action was added above in the from_range_call() call for the child call.
                    let thread_duration = just_after(&child_call.end_instant.unwrap().duration_since(first_instant));
                    action_list.push(Action::new(thread_duration, thread_number, child_call, ActionType::move_to_thread()));
                }

                // if index == 0 || child_call.is_new_thread {
                    // Either this was the first child call in the same thread or it's a child call in a new thread, or
                    // both. Take back ownership of the current range just after the later of the child calls ends.
                    // let release_duration = child_call.end_instant.unwrap().duration_since(first_instant);
                    // action_list.push(Action::new(release_duration, thread_number, child_call, ActionType::release()));
                    // Call the more elaborate constructor for Action because we need a mixture of fields from the
                    // current call and the child call.
                    // action_list.push(Action::new_detail(after_latest_end_duration, thread_number, &range_call.key, range_call.actor, child_call.from, child_call.to(), ActionType::take()));
                // }
            }
            action_list.push(Action::new(after_latest_end_duration, thread_number, range_call, ActionType::Take));

        }
        action_list.push(Action::new(end_duration, thread_number, range_call, ActionType::Release));
    }




*/