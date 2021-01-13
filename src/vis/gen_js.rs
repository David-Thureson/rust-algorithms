#![allow(dead_code)]

use crate::sort::test_data;
use super::quicksort::*;
use super::model::*;
use std::fs;
use rand::rngs::ThreadRng;
use std::collections::BTreeMap;
use std::ops::Range;
use std::fmt::Display;
use itertools::Itertools;

const FILE_JS_CONSTANTS: &str = "constants.js";
const FILE_JS_DATA: &str = "sort_data.js";

pub fn main() {
    //try_gen_one();
    try_gen_multiple_1();
}

pub enum ThreadLayout {
    Horizontal,
    Vertical,
}

pub struct JSAction {
    time: u128,
    entry: String,
}

pub fn gen_js(action_lists: &[ActionList]) {

    // action_list.sort_unstable_by_key(|x| x.duration);

    let max_item_count = action_lists
        .iter()
        .map(|list| list.unsorted_array.as_ref().unwrap().len())
        .max()
        .unwrap();

    let max_duration_nanos = action_lists
        .iter()
        .map(|list| list.actions.iter().map(|action| action.duration).max().unwrap())
        .max()
        .unwrap()
        .as_nanos();

    let mut js_items = vec![];
    let mut js_actions = vec![];
    let mut index_offset = 0;
    for (run_number, action_list) in action_lists.iter().enumerate() {
        add_js_entries(&mut js_items, &mut js_actions, action_list, run_number, max_duration_nanos, index_offset);
        index_offset += action_list.unsorted_array.as_ref().unwrap().len();
    }

    /*
    let max_values = action_lists
        .iter()
        .map(|list| list.unsorted_array.unwrap().max())
        .collect();
    */

    let mut s = fs::read_to_string(FILE_JS_CONSTANTS).unwrap();

    s.push_str("\nvar run_list = [");
    for action_list in action_lists {
        let size = action_list.unsorted_array.as_ref().unwrap().len();
        s.push_str(&format!("\n\t{{ size: {}, call_segments: [{}] }},", size, size));
    }
    s.push_str("\n];\n");

    s.push_str(&format!("\nvar max_item_count = {};\n", max_item_count));

    s.push_str(&format!("\nvar dataset = ["));
    for item in js_items.iter() {
        s.push_str(&format!("\n\t{}", item));
    }
    s.push_str(&format!("\n];\n"));

    s.push_str(&format!("\nvar steps = ["));
    for action in js_actions.iter().sorted_by_key(|action| action.time) {
        s.push_str(&format!("\n\t{}", action.entry));
    }
    s.push_str(&format!("\n];\n"));

    write_js_file(&s);
}

fn add_js_entries(js_items: &mut Vec<String>,
                  js_actions: &mut Vec<JSAction>,
                  action_list: &ActionList,
                  run_number: usize,
                  max_duration_nanos: u128,
                  index_offset: usize) {

    let v = action_list.unsorted_array.as_ref().unwrap();
    let max_value = v.iter().map(|x| *x).max().unwrap() as f64;
    for (index, val) in v.iter().enumerate() {
        let val = * val as f64 / max_value;
        js_items.push(format! ("{{ key: {}, value: {}, run_number: {}, pos: {} }},", index + index_offset, val, run_number, index));
    }

    for action in action_list.actions.iter() {
        js_actions.push(action_to_js(action, run_number, max_duration_nanos, index_offset));
    }

}

/*
fn add_actions_horizontal_threads(s: &mut String, action_list: &ActionList, _array_length: usize, max_time: u128) {
    //let mut live_threads = LiveThreadList::new(0..array_length);
    for action in action_list.iter() {


        s.push_str(&format!("\n    {}", action_to_js(action, max_time)));
    }
}
*/
fn write_js_file(content: &str) {
    fs::write(FILE_JS_DATA, content).unwrap();
    println!("{}", content);
}

fn actor_to_js(actor: &Actor) -> &str {
    match actor {
        Actor::BubbleSort => "BUBBLE_SORT",
        Actor::Partition => "PARTITION",
        Actor::Quicksort => "QUICKSORT",
    }
}

/*
pub duration: time::Duration,
pub thread_number: u8,
pub call_key: String,
pub actor: Actor,
pub from: usize,
pub to: usize,
pub action_type: ActionType,
*/

fn action_to_js(action: &Action, run_number: usize, max_duration_nanos: u128, index_offset: usize) -> JSAction {
    let action_duration_nanos = action.duration.as_nanos();
    let time_fraction = action_duration_nanos as f32 / max_duration_nanos as f32;
    let prefix = {
        let time = format!("time: {}", time_fraction);
        let run_number = format!(", run_number: {}", run_number);
        format!("{{ {}{}, action: action.", time, run_number)
    };
    let actor = format!(", actor: actor.{}", actor_to_js(&action.actor));
    let from_to = format!(", from: {}, to: {}", action.from + index_offset, action.to + index_offset);
    let suffix =  " },";
    let entry = match &action.action_type {
        ActionType::MoveToThread => format!("{}MOVE_TO_THREAD, thread_number: {}{}{}", prefix, action.thread_number, from_to, suffix),
        ActionType::MoveToCall { from_call_key: _ , to_call_key: _, ranges} => {
            let mut s = String::new();
            s.push_str(&format!("{}MOVE_TO_CALL, call_segments: [", prefix));
            for range in ranges {
                // s.push_str(&format!("{{ from: {}, to: {} }}, ", range.start, range.end));
                s.push_str(&format!("{}, ", range.end));
            }
            s.push_str(&format!("]{}", suffix));
            s
        }
        ActionType::Take => format!("{}TAKE{}{}{}", prefix, actor, from_to, suffix),
        ActionType::Release => format!("{}RELEASE{}{}", prefix, from_to, suffix),
        ActionType::Swap { a, b } => format!("{}SWAP, a: {}, b: {}{}", prefix, a + index_offset, b + index_offset, suffix),
        ActionType::MarkFinal{ from, to } => format!("{}MARK_FINAL, from: {}, to: {}{}", prefix, from + index_offset, to + index_offset, suffix),
    };
    JSAction {
        time: action_duration_nanos,
        entry,
    }
}

/*
fn try_gen_one() {
    let thread_layout = ThreadLayout::Vertical;
    let label = "Linux_500_20_1";
    let min_split_size = 20;
    let max_threads = 1;
    let size = 500;
    // let min_split_size = 10;
    // let max_threads = 2;
    // let size = 100;
    let mut v = test_data::vec_usize_shuffled(size);
    let unsorted_array = v.clone();
    dbg!(&v);
    let include_detail_actions = true;
    let range_call = quicksort(&mut v, min_split_size, max_threads);
    range_call.try_log(include_detail_actions);
    let action_list = ActionList::from_range_call(&range_call, Some(label));
    gen_js(&unsorted_array, action_list]);
}
*/

fn try_gen_multiple_1() {
    let size = 150;
    let thread_splits = 0;
    let mut action_lists = vec![];
    let v =        test_data::vec_usize_shuffled(size);
    for min_split_size in [10, 20, 40, 80, 160].iter() {
        action_lists.push(make_action_list(size, *min_split_size as usize, thread_splits, Some(v.clone())));
    }
    // gen_js(&action_lists);
}

fn try_gen_multiple_2() {
    let size = 500;
    let min_split_size = 20;
    let mut action_lists = vec![];
    let v =        test_data::vec_usize_shuffled(size);
    for thread_splits in 0..=5 {
        action_lists.push(make_action_list(size, min_split_size as usize, thread_splits, Some(v.clone())));
    }
    gen_js(&action_lists);
}

fn make_action_list(size: usize, min_split_size: usize, thread_splits: u8, shared_v: Option<Vec<usize>>) -> ActionList {
    let label = &format!("Linux: count = {}; simple sort threshold = {}; max thread splits = {}", size, min_split_size, thread_splits);
    let max_threads = 2u8.pow(thread_splits.into());
    let mut v = if let Some(shared_v) = shared_v {
        shared_v.clone()
    } else {
        test_data::vec_usize_shuffled(size)
    };
    let unsorted_array = v.clone();
    let range_call = quicksort(&mut v, min_split_size, max_threads);
    // range_call.try_log(include_detail_actions);
    let mut action_list = ActionList::from_range_call(&range_call, Some(label));

    action_list.report_types();

    action_list.unsorted_array = Some(unsorted_array);
    action_list
}
