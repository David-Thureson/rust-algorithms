use std::fmt::{self, Debug};
use std::sync::Mutex;
use std::time;
use util::format;
use itertools::Itertools;
use std::cmp::Ordering;
use std::time::Instant;

pub fn main() {

}

#[derive(Clone)]
// #[derive(Debug)]
pub struct Counter<T: Debug + Clone> {
    // pub items: Mutex<Vec<CounterItem<T>>>,
    pub items: Vec<CounterItem<T>>,
    // pub child_sort_function: Box<dyn FnMut((usize, &CounterItem<T>), (usize, &CounterItem<T>)) -> Option<Ordering>>,
}

#[derive(Clone)]
pub struct CounterItem<T: Debug + Clone> {
    pub start: time::Instant,
    pub end: Option<time::Instant>,
    pub size: usize,
    pub is_new_thread: bool,
    pub data: Option<T>,
    pub parent_index: Option<usize>,
}

impl <T: Debug + Clone> Counter<T> {

    pub fn new() -> Self {
        Counter {
            items: vec![],
            // child_sort_function: |((a_index, _), (b_index, _))| a_index.partial_cmp(b_index),
        }
    }

    pub fn start(&mut self, size: usize, data: Option<T>, is_new_thread: bool, parent_index: Option<usize>) -> usize {
        // let items = self.items.get_mut().unwrap();
        self.items.push(CounterItem {
            start: time::Instant::now(),
            end: None,
            size,
            is_new_thread,
            data,
            parent_index,
        });
        self.items.len() - 1
    }

    pub fn end(&mut self, index: usize) {
        // let items = self.items.get_mut().unwrap();
        self.items.get_mut(index).unwrap().end = Some(time::Instant::now());
    }

    // FnMut(&T, &T) -> Ordering
    pub fn describe_deep(&self, max_depth: Option<usize>, child_sort_function: &dyn Fn(&(usize, &CounterItem<T>), &(usize, &CounterItem<T>)) -> Ordering) -> String {
        let mut s = "".to_string();
        self.describe_deep_internal(&mut s, 0, 0, max_depth, child_sort_function);
        s
    }

    fn describe_deep_internal(&self, s: &mut String, index: usize, depth: usize, max_depth: Option<usize>, child_sort_function: &dyn Fn(&(usize, &CounterItem<T>), &(usize, &CounterItem<T>)) -> Ordering) {
        let item = self.items.get(index).unwrap();
        let line = format::format_indent_space(depth, &format!("[{}] {}", index, format!("{:?}", item)));
        s.push_str(&format!("\n{}", line));
        if max_depth.is_none() || max_depth.unwrap() > depth {
            let mut child_items: Vec<(usize, &CounterItem<T>)> = self.items.iter()
                .enumerate()
                .filter(|(_index, item)| item.parent_index == Some(index))
                .collect();
            child_items.sort_unstable_by(child_sort_function);
            for (index, _item) in child_items {
                self.describe_deep_internal(s, index, depth + 1, max_depth, child_sort_function);
            }
            /*
            self.items.iter()
                .enumerate()
                .filter(|(_, item)| item.parent_index == Some(index))
                .sorted_by(child_sort_function)
                .for_each(|(index, _)| {
                    self.describe_deep_internal(s, index, depth + 1, max_depth, child_sort_function);
                })
                */
        }
    }

}

impl <T: Debug + Clone> Debug for Counter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let child_sort_function//: dyn Fn(&(usize, &CounterItem<T>), &(usize, &CounterItem<T>)) -> Ordering
            = |a: &(usize, &CounterItem<T>), b: &(usize, &CounterItem<T>)| a.0.cmp(&b.0);
        // let child_sort_function// : dyn Fn(&(usize, &CounterItem<T>), &(usize, &CounterItem<T>)) -> Option<Ordering>
        //     = |a, b| a.0.partial_cmp(&b.0);
        write!(f, "{}", self.describe_deep(None, &child_sort_function))
    }
}

impl <T: Debug + Clone> Debug for CounterItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_new_thread = if self.is_new_thread { " T".to_string() } else { "  ".to_string() };
        let size = format!(" size = {}", util::format::format_count(self.size));
        let elapsed = format!("; {}", describe_elapsed(self.start, self.end, Some(self.size), Some("item")));
        let data = self.data.as_ref().map_or("".to_string(), |x| format!("; data = {:?}", &x));
        let line = format!("{}{}{}{}", is_new_thread, size, elapsed, data);
        write!(f, "{}", line)
    }
}

fn describe_elapsed(start: Instant, end: Option<Instant>, count: Option<usize>, item_label: Option<&str>) -> String {
    match end {
        Some(end) => {
            let microseconds =  end.duration_since(start).as_micros();
            let mut s = format!("elapsed = {} µs", util::format::format_count(microseconds));
            if let Some(count) = count {
                if count > 0 {
                    let nanoseconds =  end.duration_since(start).as_nanos();
                    let nano_per_item = nanoseconds / count as u128;
                    s.push_str(&format!(" ({} ns/{})", nano_per_item, item_label.unwrap_or("item")));
                }
            }
            s
        },
        None => " {no end time}".to_string()
    }
}
