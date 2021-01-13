use std::fmt::Display;
use std::ops::{Range, AddAssign, Sub};
use std::usize;

use util::format;
use itertools::Itertools;
use num_format::ToFormattedStr;

#[derive(Clone)]
pub struct RangeSet
{
    pub allow_gaps: bool,
    pub merge_adds: bool,
    pub full_range: Option<Range<usize>>,
    ranges: Vec<Range<usize>>,
}

impl RangeSet
{
    pub fn new(allow_gaps: bool, merge_adds: bool, full_range: Option<Range<usize>>) -> Self {
        RangeSet {
            allow_gaps,
            merge_adds,
            full_range,
            ranges: vec![],
        }
    }

    pub fn add_range(&mut self, other: &Range<usize>) {
        assert!(!self.ranges.contains(other));
        if let Some(full_range) = &self.full_range {
            assert!(other.start >= full_range.start);
            assert!(other.start < full_range.end);
            assert!(other.end > full_range.start);
            assert!(other.end <= full_range.end);
        }
        assert!(!self.ranges.iter().any(|range| range.contains(&other.start)));
        assert!(!self.ranges.iter().any(|range| other.contains(&range.start)));
        if self.merge_adds {
            let mut working_start = other.start;
            let mut working_end = other.end;
            let ws = working_start;
            let we = working_end;
            for found_range in self.ranges.drain_filter(|found_range| found_range.end == ws) {
                working_start = found_range.start;
            }
            for found_range in self.ranges.drain_filter(|found_range| we == found_range.start) {
                working_end = found_range.end;
            }
            self.ranges.push(working_start..working_end);
        } else {
            self.ranges.push(other.start..other.end);
        }
        debug_assert!(self.check_ranges(false, Some("after add_range()"), None));
    }

    pub fn subtract_range(&mut self, other: &Range<usize>) {
        let mut found_count = 0;
        let mut ranges_to_add = vec![];
        for found_range in self.ranges
            .drain_filter(|found_range| found_range.start <= other.start && found_range.end >= other.end) {
            if other.start > found_range.start {
                ranges_to_add.push(found_range.start..other.start);
            }
            if other.end < found_range.end {
                ranges_to_add.push(other.end..found_range.end);
            }
            found_count += 1;
        }
        assert_eq!(found_count, 1);
        for range in ranges_to_add {
            self.ranges.push(range.start..range.end);
        }
        debug_assert!(self.check_ranges(false, Some("after subtract_range()"), None));
    }

    pub fn ranges_in_order(&self) -> Vec<&Range<usize>> {
        self.ranges.iter().sorted_by_key(|x| x.start).map(|x| x).collect()
    }

    pub fn check_ranges(&self, always_show: bool, label: Option<&str>, print_limit: Option<usize>) -> bool {
        let error = !self.ranges_are_complete();
        if always_show || error {
            let (gaps, overlaps, out_of_bounds) = self.get_gaps_and_overlaps();
            let label = if let Some(label) = label {
                format!(" for {}", label)
            } else {
                "".to_string()
            };
            println!("RangeSet.check_ranges{}:", label);
            if gaps.len() > 0 {
                Self::print_ranges(1, &gaps, Some("Gaps"), None);
            }
            if overlaps.len() > 0 {
                Self::print_ranges(1, &overlaps, Some("Overlaps"), None);
            }
            if out_of_bounds.len() > 0 {
                Self::print_ranges(1, &out_of_bounds, Some("Out of Bounds"), None);
            }
            self.print_deep(1, Some("All Ranges"), print_limit);
        }
        if error {
            panic!();
        }
        true
    }

    pub fn ranges_are_complete(&self) -> bool {
        let (gaps, overlaps, out_of_bounds) = self.get_gaps_and_overlaps();
        let gaps_len_for_error = if self.allow_gaps { 0 } else { gaps.len() };
        gaps_len_for_error + overlaps.len() + out_of_bounds.len() == 0
    }

    pub fn get_gaps_and_overlaps(&self) -> (Vec<Range<usize>>, Vec<Range<usize>>, Vec<Range<usize>>) {
        let mut gaps = vec![];
        let mut overlaps = vec![];
        let mut out_of_bounds= vec![];
        let mut prev_range_opt: Option<Range<usize>> = None;
        for (index, range) in self.ranges_in_order().iter().enumerate() {
            if index == 0 {
                if let Some(full_range) = &self.full_range {
                    if range.start < full_range.start {
                        out_of_bounds.push(range.start..range.end);
                    } else if range.start > full_range.start {
                        gaps.push(full_range.start..range.start);
                    }
                }
            } else {
                let prev_end = prev_range_opt.unwrap().end;
                if range.start < prev_end {
                    overlaps.push(range.start..prev_end);
                } else if range.start > prev_end {
                    gaps.push(prev_end..range.start);
                }
            }
            prev_range_opt = Some(range.start..range.end);
        }
        if let Some(full_range) = &self.full_range {
            if let Some(prev_range) = prev_range_opt {
                if prev_range.end < full_range.end {
                    gaps.push(prev_range.end..full_range.end);
                } else if prev_range.end > full_range.end {
                    out_of_bounds.push(prev_range);
                }
            }
        }
        (gaps, overlaps, out_of_bounds)
    }

    pub fn print_deep(&self, depth: usize, label: Option<&str>, limit: Option<usize>) {
        let limit_note = if let Some(limit) = limit {
            if limit < self.ranges.len() { format!(" (limited to {}", format::format_count(limit)) } else { "".to_string() }
        } else {
            "".to_string()
        };
        let label = &format!("{}{}:", label.unwrap_or("RangeSet"), limit_note);
        Self::print_ranges(depth, &self.ranges, Some(label), limit);
    }

    pub fn print_ranges(mut depth: usize, ranges: &Vec<Range<usize>>, label: Option<&str>, limit: Option<usize>) {
        let limit = limit.unwrap_or(usize::MAX);
        if let Some(label) = label {
            format::println_indent_tab(depth, &format!("{}:", label));
            depth += 1;
        }
        let max_end_width = ranges.iter().take(limit).map(|x| format::format_count(x.end).len()).max().unwrap();
        let max_length_width = ranges.iter().take(limit).map(|x| format::format_count(x.end - x.start).len()).max().unwrap();
        for range in ranges.iter().take(limit) {
            format::println_indent_tab(depth, &format!("[{:>end_width$}, {:>end_width$}): {:>length_width$}",
                                                       range.start, range.end, range.end - range.start,
                                                       end_width = max_end_width, length_width = max_length_width));
        }
    }
}


