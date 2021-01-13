#![allow(dead_code)]

use std::{thread, time};
use std::sync::atomic;
use std::mem;
use std::fmt::{self, Debug};

use crate::*;
use std::sync::{Arc, Mutex};
use std::borrow::BorrowMut;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::cell::RefCell;

static CALL_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
static THREAD_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
static LIVE_THREAD_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

// const SLEEP_NANOSECONDS_PER_ITEM: isize = 88;
const MIN_SPLIT_SIZE: usize = 10;
const NSEC_PER_ITEM: u64 = 100;

pub fn main() {
    // let nsec_per_item = 100;
    // let nsec_per_item = 0;
    // try_divide_simple_defer_high_low(nsec_per_item, true);
    // try_divide_simple_defer_high_low(nsec_per_item, false);
    // try_few_parms();
    // try_divide_simple_parallel(100, true);
    // try_divide_parallel_settings();
    // try_atomic_counter();
    // try_atomic_counter_orderings();
    // try_mutex_and_no_return();
    try_monitor();
    // get_sizes();
}

pub fn divide_simple_defer_high(low: usize, high: usize, min_split_size: usize, nsec_per_item: u64, inline_fake_work: bool) -> usize {
    1
        + if high - low >= min_split_size {
            let mid = (low + high) / 2;
            divide_simple_defer_high(low,mid, min_split_size, nsec_per_item, inline_fake_work)
                + divide_simple_defer_high(mid, high, min_split_size, nsec_per_item, inline_fake_work)
        } else if inline_fake_work {
            fake_work_inline(low, high, nsec_per_item)
        } else {
            fake_work(low, high, nsec_per_item)
        }
}

pub fn divide_simple_defer_low(low: usize, high: usize, min_split_size: usize, nsec_per_item: u64, inline_fake_work: bool) -> usize {
    let mid = (low + high) / 2;
    1
        + if mid - low >= min_split_size {
            divide_simple_defer_low(low, mid, min_split_size, nsec_per_item, inline_fake_work)
        } else if inline_fake_work {
            fake_work_inline(low, mid, nsec_per_item)
        } else {
            fake_work(low, mid, nsec_per_item)
        }
        + if high - mid >= min_split_size {
            divide_simple_defer_low(mid, high, min_split_size, nsec_per_item, inline_fake_work)
        } else if inline_fake_work {
            fake_work_inline(mid, high, nsec_per_item)
        } else {
            fake_work(mid, high, nsec_per_item)
        }
}

pub fn divide_simple_defer_low_few_parms(low: usize, high: usize) -> usize {
    let mid = (low + high) / 2;
    1
        + if mid - low >= MIN_SPLIT_SIZE {
            divide_simple_defer_low_few_parms(low, mid)
        } else {
            fake_work_inline_few_parms(low, mid)
        }
        + if high - mid >= MIN_SPLIT_SIZE {
            divide_simple_defer_low_few_parms(mid, high)
        } else {
            fake_work_inline_few_parms(mid, high)
        }
}

pub fn divide_simple_parallel(low: usize, high: usize, min_split_size: usize, min_thread_size: usize, nsec_per_item: u64, inline_fake_work: bool, use_atomic_counter: bool) -> (usize, usize) {
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let mut first_call_count = None;
    let mut first_thread_count = None;
    let handle = if first_len >= min_thread_size {
        Some(thread::spawn(move || {
            divide_simple_parallel(low, mid, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter)
        }))
    } else if first_len >= min_split_size {
        let (call_ct, thread_ct) = divide_simple_parallel(low, mid, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter);
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct);
        None
    } else {
        let call_ct = if inline_fake_work {
            fake_work_inline(low, mid, nsec_per_item)
        } else {
            fake_work(low, mid, nsec_per_item)
        };
        first_call_count = Some(call_ct);
        first_thread_count = Some(0);
        None
    };
    let (second_call_count, second_thread_count) = if high - mid >= min_split_size {
        divide_simple_parallel(low, mid, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter)
    } else {
        if inline_fake_work {
            (fake_work_inline(mid, high, nsec_per_item), 0)
        } else {
            (fake_work(mid, high, nsec_per_item), 0)
        }
    };

    if let Some(handle) = handle {
        let (call_ct, thread_ct) = handle.join().unwrap();
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct + 1);
    }

    (1 + first_call_count.unwrap() + second_call_count, first_thread_count.unwrap() + second_thread_count)
}

#[derive(Copy, Clone)]
pub struct DivideParallelSettings {
    min_split_size: usize,
    min_thread_size: usize,
    nsec_per_item: u64,
    inline_fake_work: bool,
    use_atomic_counter: bool,
}

#[derive(Copy, Clone)]
pub struct DivideParallelSettings2 {
    min_split_size: u8,
    min_thread_size: u32,
    nsec_per_item: u8,
    inline_fake_work: bool,
    atomic_counter_ordering: Option<atomic::Ordering>,
}

#[derive(Copy, Clone)]
pub struct DivideParallelSettingsNoReturn {
    min_split_size: u8,
    min_thread_size: u32,
    nsec_per_item: u8,
    inline_fake_work: bool,
    ordering: atomic::Ordering,
}

#[derive(Copy, Clone)]
pub struct DivideParallelSettingsMonitor {
    min_split_size: u8,
    min_thread_size: u32,
    nsec_per_item: u8,
    ordering: atomic::Ordering,
}

#[derive(Copy, Clone)]
pub struct DivideParallelSettingsMutex {
    min_split_size: u8,
    min_thread_size: u32,
    nsec_per_item: u8,
    inline_fake_work: bool,
}

/*
#[derive(Copy, Clone)]
pub struct DivideParallelSettings2b {
    min_split_size: u8,
    min_thread_size: u32,
    nsec_per_item: u8,
    inline_fake_work: bool,
    use_atomic_counter: bool,
    ordering: atomic::Ordering,
}

#[derive(Copy, Clone)]
pub struct DivideParallelSettings2c {
    min_split_size: u8,
    min_thread_size: u32,
    nsec_per_item: u8,
    inline_fake_work: bool,
    ordering: Option<atomic::Ordering>,
}
*/

#[derive(Copy, Clone)]
pub struct DivideParallelSettings3 {
    min_split_size: u32,
    min_thread_size: u32,
    nsec_per_item: u8,
    inline_fake_work: bool,
    use_atomic_counter: bool,
}

pub fn divide_parallel_settings(low: usize, high: usize, min_split_size: usize, min_thread_size: usize, nsec_per_item: u64, inline_fake_work: bool, use_atomic_counter: bool) -> (usize, usize) {
    divide_parallel_settings_internal(low, high, DivideParallelSettings {
        min_split_size,
        min_thread_size,
        nsec_per_item,
        inline_fake_work,
        use_atomic_counter,
    })
}

fn divide_parallel_settings_internal(low: usize, high: usize, settings: DivideParallelSettings) -> (usize, usize) {
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let mut first_call_count = None;
    let mut first_thread_count = None;
    let handle = if first_len >= settings.min_thread_size {
        Some(thread::spawn(move || {
            divide_parallel_settings_internal(low, mid, settings)
        }))
    } else if first_len >= settings.min_split_size {
        let (call_ct, thread_ct) = divide_parallel_settings_internal(low, mid, settings);
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct);
        None
    } else {
        let call_ct = if settings.inline_fake_work {
            fake_work_inline(low, mid, settings.nsec_per_item)
        } else {
            fake_work(low, mid, settings.nsec_per_item)
        };
        first_call_count = Some(call_ct);
        first_thread_count = Some(0);
        None
    };
    let (second_call_count, second_thread_count) = if high - mid >= settings.min_split_size {
        divide_parallel_settings_internal(low, mid, settings)
    } else {
        if settings.inline_fake_work {
            (fake_work_inline(mid, high, settings.nsec_per_item), 0)
        } else {
            (fake_work(mid, high, settings.nsec_per_item), 0)
        }
    };

    if let Some(handle) = handle {
        let (call_ct, thread_ct) = handle.join().unwrap();
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct + 1);
    }

    (1 + first_call_count.unwrap() + second_call_count, first_thread_count.unwrap() + second_thread_count)
}

pub fn divide_parallel_settings_2(low: u32, high: u32, min_split_size: u8, min_thread_size: u32, nsec_per_item: u8, inline_fake_work: bool, atomic_counter_ordering: Option<atomic::Ordering>) -> (u32, u32) {
    divide_parallel_settings_internal_2(low, high, DivideParallelSettings2 {
        min_split_size,
        min_thread_size,
        nsec_per_item,
        inline_fake_work,
        atomic_counter_ordering,
    })
}

fn divide_parallel_settings_internal_2(low: u32, high: u32, settings: DivideParallelSettings2) -> (u32, u32) {
    let min_thread_size = settings.min_thread_size as u32;
    let min_split_size = settings.min_split_size as u32;
    let mut internal_call_count = 1;
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let mut first_call_count = None;
    let mut first_thread_count = None;
    let handle = if first_len >= min_thread_size {
        Some(thread::spawn(move || {
            divide_parallel_settings_internal_2(low, mid, settings)
        }))
    } else if first_len >= min_split_size {
        let (call_ct, thread_ct) = divide_parallel_settings_internal_2(low, mid, settings);
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct);
        None
    } else {
        internal_call_count += 1;
        let call_ct = if settings.inline_fake_work {
            fake_work_inline_2(low, mid, settings.nsec_per_item)
        } else {
            fake_work_2(low, mid, settings.nsec_per_item)
        };
        first_call_count = Some(call_ct);
        first_thread_count = Some(0);
        None
    };
    let (second_call_count, second_thread_count) = if high - mid >= min_split_size {
        divide_parallel_settings_internal_2(low, mid, settings)
    } else {
        internal_call_count += 1;
        if settings.inline_fake_work {
            (fake_work_inline_2(mid, high, settings.nsec_per_item), 0)
        } else {
            (fake_work_2(mid, high, settings.nsec_per_item), 0)
        }
    };

    if let Some(handle) = handle {
        let (call_ct, thread_ct) = handle.join().unwrap();
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct + 1);
    }

    if let Some(ordering) = settings.atomic_counter_ordering {
        CALL_COUNTER.fetch_add(internal_call_count, ordering);
    }

    (1 + first_call_count.unwrap() + second_call_count, first_thread_count.unwrap() + second_thread_count)
}

pub fn divide_parallel_no_return(low: u32, high: u32, min_split_size: u8, min_thread_size: u32, nsec_per_item: u8, inline_fake_work: bool, ordering: atomic::Ordering) {
    divide_parallel_no_return_internal(low, high, DivideParallelSettingsNoReturn {
        min_split_size,
        min_thread_size,
        nsec_per_item,
        inline_fake_work,
        ordering,
    })
}

fn divide_parallel_no_return_internal(low: u32, high: u32, settings: DivideParallelSettingsNoReturn) {
    let min_thread_size = settings.min_thread_size as u32;
    let min_split_size = settings.min_split_size as u32;
    let mut internal_call_count = 1;
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let handle = if first_len >= min_thread_size {
        THREAD_COUNTER.fetch_add(1, settings.ordering);
        Some(thread::spawn(move || {
            divide_parallel_no_return_internal(low, mid, settings)
        }))
    } else if first_len >= min_split_size {
        divide_parallel_no_return_internal(low, mid, settings);
        None
    } else {
        internal_call_count += 1;
        if settings.inline_fake_work {
            fake_work_inline_2(low, mid, settings.nsec_per_item);
        } else {
            fake_work_2(low, mid, settings.nsec_per_item);
        };
        None
    };
    if high - mid >= min_split_size {
        divide_parallel_no_return_internal(low, mid, settings);
    } else {
        internal_call_count += 1;
        if settings.inline_fake_work {
            fake_work_inline_2(mid, high, settings.nsec_per_item);
        } else {
            fake_work_2(mid, high, settings.nsec_per_item);
        }
    };

    if let Some(handle) = handle {
        handle.join().unwrap();
    }

    CALL_COUNTER.fetch_add(internal_call_count, settings.ordering);
}

pub fn divide_parallel_monitor(low: u32, high: u32, min_split_size: u8, min_thread_size: u32, nsec_per_item: u8, ordering: atomic::Ordering) {
    divide_parallel_monitor_internal(low, high, DivideParallelSettingsMonitor {
        min_split_size,
        min_thread_size,
        nsec_per_item,
        ordering,
    })
}

fn divide_parallel_monitor_internal(low: u32, high: u32, settings: DivideParallelSettingsMonitor) {
    let min_thread_size = settings.min_thread_size as u32;
    let min_split_size = settings.min_split_size as u32;
    let mut internal_call_count = 1;
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let handle = if first_len >= min_thread_size {
        THREAD_COUNTER.fetch_add(1, settings.ordering);
        LIVE_THREAD_COUNTER.fetch_add(1, settings.ordering);
        Some(thread::spawn(move || {
            divide_parallel_monitor_internal(low, mid, settings)
        }))
    } else if first_len >= min_split_size {
        divide_parallel_monitor_internal(low, mid, settings);
        None
    } else {
        internal_call_count += 1;
        fake_work_inline_2(low, mid, settings.nsec_per_item);
        None
    };
    if high - mid >= min_split_size {
        divide_parallel_monitor_internal(low, mid, settings);
    } else {
        internal_call_count += 1;
        fake_work_inline_2(mid, high, settings.nsec_per_item);
    };

    if let Some(handle) = handle {
        handle.join().unwrap();
        LIVE_THREAD_COUNTER.fetch_sub(1, settings.ordering);
    }

    CALL_COUNTER.fetch_add(internal_call_count, settings.ordering);
}

pub fn divide_parallel_mutex(low: u32, high: u32, min_split_size: u8, min_thread_size: u32, nsec_per_item: u8, inline_fake_work: bool) -> (u32, u32, u32) {
    let settings =  DivideParallelSettingsMutex {
        min_split_size,
        min_thread_size,
        nsec_per_item,
        inline_fake_work,
    };
    let counter = Arc::new(Mutex::new(Counter::new()));
    let (call_count, thread_count) = divide_parallel_mutex_internal(low, high, settings, Arc::clone(&counter));
    let counter_call_count = counter.lock().unwrap().call_count;
    (call_count, thread_count, counter_call_count)
}

fn divide_parallel_mutex_internal(low: u32, high: u32, settings: DivideParallelSettingsMutex, counter: Arc<Mutex<Counter>>) -> (u32, u32) {
    let min_thread_size = settings.min_thread_size as u32;
    let min_split_size = settings.min_split_size as u32;
    let mut internal_call_count = 1;
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let mut first_call_count = None;
    let mut first_thread_count = None;
    let first_counter = Arc::clone(&counter);
    let handle = if first_len >= min_thread_size {
        Some(thread::spawn(move || {
            divide_parallel_mutex_internal(low, mid, settings, first_counter)
        }))
    } else if first_len >= min_split_size {
        let (call_ct, thread_ct) = divide_parallel_mutex_internal(low, mid, settings, first_counter);
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct);
        None
    } else {
        internal_call_count += 1;
        let call_ct = if settings.inline_fake_work {
            fake_work_inline_2(low, mid, settings.nsec_per_item)
        } else {
            fake_work_2(low, mid, settings.nsec_per_item)
        };
        first_call_count = Some(call_ct);
        first_thread_count = Some(0);
        None
    };
    let (second_call_count, second_thread_count) = if high - mid >= min_split_size {
        divide_parallel_mutex_internal(low, mid, settings, Arc::clone(&counter))
    } else {
        internal_call_count += 1;
        if settings.inline_fake_work {
            (fake_work_inline_2(mid, high, settings.nsec_per_item), 0)
        } else {
            (fake_work_2(mid, high, settings.nsec_per_item), 0)
        }
    };

    if let Some(handle) = handle {
        let (call_ct, thread_ct) = handle.join().unwrap();
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct + 1);
    }

    {
        counter.lock().unwrap().inc(internal_call_count);
    }

    (1 + first_call_count.unwrap() + second_call_count, first_thread_count.unwrap() + second_thread_count)
}

pub fn divide_parallel_settings_3(low: u32, high: u32, min_split_size: u32, min_thread_size: u32, nsec_per_item: u8, inline_fake_work: bool, use_atomic_counter: bool) -> (u32, u32) {
    divide_parallel_settings_internal_3(low, high, DivideParallelSettings3 {
        min_split_size,
        min_thread_size,
        nsec_per_item,
        inline_fake_work,
        use_atomic_counter,
    })
}

fn divide_parallel_settings_internal_3(low: u32, high: u32, settings: DivideParallelSettings3) -> (u32, u32) {
    let mid = (low + high) / 2;
    let first_len = mid - low;
    let mut first_call_count = None;
    let mut first_thread_count = None;
    let handle = if first_len >= settings.min_thread_size {
        Some(thread::spawn(move || {
            divide_parallel_settings_internal_3(low, mid, settings)
        }))
    } else if first_len >= settings.min_split_size {
        let (call_ct, thread_ct) = divide_parallel_settings_internal_3(low, mid, settings);
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct);
        None
    } else {
        let call_ct = if settings.inline_fake_work {
            fake_work_inline_2(low, mid, settings.nsec_per_item)
        } else {
            fake_work_2(low, mid, settings.nsec_per_item)
        };
        first_call_count = Some(call_ct);
        first_thread_count = Some(0);
        None
    };
    let (second_call_count, second_thread_count) = if high - mid >= settings.min_split_size {
        divide_parallel_settings_internal_3(low, mid, settings)
    } else {
        if settings.inline_fake_work {
            (fake_work_inline_2(mid, high, settings.nsec_per_item), 0)
        } else {
            (fake_work_2(mid, high, settings.nsec_per_item), 0)
        }
    };

    if let Some(handle) = handle {
        let (call_ct, thread_ct) = handle.join().unwrap();
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct + 1);
    }

    (1 + first_call_count.unwrap() + second_call_count, first_thread_count.unwrap() + second_thread_count)
}

struct SettingsForGeneric {
    min_split_size: u8,
    nsec_per_item: u8,
}

/*
pub fn divide_generic<L, S>(low: L, size: S, min_split_size: u8, nsec_per_item: u8, update_counter: bool) -> Option<CounterForGeneric>
    where
        L: Into<u32>,
        S: Into<u32>,
{
    let settings = SettingsForGeneric {
        min_split_size,
        nsec_per_item,
    };
    let counter= if update_counter {
        Some(Rc::new(RefCell::new(CounterForGeneric::new())))
    } else {
        None
    };
    let counter_clone = counter.map(|ct| Rc::clone(&ct));
    divide_generic_internal(low, size, settings, counter_clone);
    counter.map(|ct| ct.borrow().clone())
}

fn divide_generic_internal<L, S>(low: L, size: S, settings: SettingsGeneric<L, S>, counter: Option<Rc<RefCell<CounterForGeneric>>>)
    where
        L: Into<u32>,
        S: Into<u32>,
{
    let low_u32 = settings.low as u32;
    let size_u32 = settincgs.size as u32;
    let min_split_size = settings.min_split_size as u32;
    let mut internal_call_count = 1;
    let mid = low_u32 + (size_u32 / 2);
    let first_size = mid - low_u32;
    if first_size >= min_split_size {
        divide_generic_internal(low, first_size, settings, counter.map(|ct| Rc::clone(&ct)));
        first_call_count = Some(call_ct);
        first_thread_count = Some(thread_ct);
        None
    } else {
        internal_call_count += 1;
        fake_work_inline_for_generic(size as u8, settings.nsec_per_item);
    };
    let second_size = (low_u32 + size_u32) - mid;
    if second_size >= min_split_size {
        divide_generic_internal(low, second_size, settings, counter.map(|ct| Rc::clone(&ct)));
    } else {
        internal_call_count += 1;
        internal_call_count += 1;
        fake_work_inline_for_generic(size as u8, settings.nsec_per_item);
    };

    {
        counter.lock().unwrap().inc(internal_call_count);
    }

    (1 + first_call_count.unwrap() + second_call_count, first_thread_count.unwrap() + second_thread_count)
}
*/
fn fake_work(low: usize, high: usize, nsec_per_item: u64) -> usize {
    if nsec_per_item > 0 {
        thread::sleep(time::Duration::from_nanos((high as isize - low as isize) as u64 * nsec_per_item));
    }
    1
}

#[inline]
fn fake_work_inline(low: usize, high: usize, nsec_per_item: u64) -> usize {
    if nsec_per_item > 0 {
        thread::sleep(time::Duration::from_nanos((high as isize - low as isize) as u64 * nsec_per_item));
    }
    1
}

fn fake_work_2(low: u32, high: u32, nsec_per_item: u8) -> u32 {
    if nsec_per_item > 0 {
        thread::sleep(time::Duration::from_nanos((high as isize - low as isize) as u64 * (nsec_per_item as u64)));
    }
    1
}

#[inline]
fn fake_work_inline_2(low: u32, high: u32, nsec_per_item: u8) -> u32 {
    if nsec_per_item > 0 {
        thread::sleep(time::Duration::from_nanos((high as isize - low as isize) as u64 * (nsec_per_item as u64)));
    }
    1
}

#[inline]
fn fake_work_inline_for_generic(size: u8, nsec_per_item: u8) {
    if nsec_per_item > 0 {
        thread::sleep(time::Duration::from_nanos(size as u64 * nsec_per_item as u64));
    }
}

#[inline]
fn fake_work_inline_few_parms(low: usize, high: usize) -> usize {
    if NSEC_PER_ITEM > 0 {
        thread::sleep(time::Duration::from_nanos((high as isize - low as isize) as u64 * NSEC_PER_ITEM));
    }
    1
}

fn try_divide_simple_defer_high_low(nsec_per_item: u64, inline_fake_work: bool) {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let min_thread_size = 1_000;
    let use_atomic_counter = false;
    format::print_elapsed(true, "divide_simple_defer_high", "", || {
        dbg!(divide_simple_defer_high(low, high, min_split_size, nsec_per_item, inline_fake_work));
    });
    format::print_elapsed(true, "divide_simple_defer_low", "", || {
        dbg!(divide_simple_defer_low(low, high, min_split_size, nsec_per_item, inline_fake_work));
    });
    format::print_elapsed(true, "divide_simple_parallel", "", || {
        dbg!(divide_simple_parallel(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter));
    });
}

fn try_few_parms() {
    let low = 0;
    let high = 1_000_000;
    format::print_elapsed(true, "divide_simple_defer_low", "", || {
        dbg!(divide_simple_defer_low(low, high, MIN_SPLIT_SIZE, NSEC_PER_ITEM, true));
    });
    format::print_elapsed(true, "divide_simple_defer_low_few_parms", "", || {
        dbg!(divide_simple_defer_low_few_parms(low, high));
    });
}

fn try_divide_simple_parallel(nsec_per_item: u64, inline_fake_work: bool) {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let use_atomic_counter = false;
    format::print_elapsed(true, "divide_simple_parallel_100", "", || {
        dbg!(divide_simple_parallel(low, high, min_split_size, 100, nsec_per_item, inline_fake_work, use_atomic_counter));
    });
    format::print_elapsed(true, "divide_simple_parallel_1_000", "", || {
        dbg!(divide_simple_parallel(low, high, min_split_size, 1_000, nsec_per_item, inline_fake_work, use_atomic_counter));
    });
    format::print_elapsed(true, "divide_simple_parallel_10_000", "", || {
        dbg!(divide_simple_parallel(low, high, min_split_size, 10_000, nsec_per_item, inline_fake_work, use_atomic_counter));
    });
    format::print_elapsed(true, "divide_simple_parallel_100_000", "", || {
        dbg!(divide_simple_parallel(low, high, min_split_size, 100_000, nsec_per_item, inline_fake_work, use_atomic_counter));
    });

}

fn try_divide_parallel_settings() {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let min_thread_size = 1_000;
    let nsec_per_item = 0;
    let inline_fake_work = true;
    let use_atomic_counter = false;
    let atomic_counter_ordering = None;
    format::print_elapsed(true, "divide_simple_parallel", "", || {
        dbg!(divide_simple_parallel(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter));
    });
    format::print_elapsed(true, "divide_parallel_settings", "", || {
        dbg!(divide_parallel_settings(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, use_atomic_counter));
    });
    format::print_elapsed(true, "divide_parallel_settings_2", "", || {
        dbg!(divide_parallel_settings_2(low as u32, high as u32, min_split_size as u8, min_thread_size as u32, nsec_per_item as u8, inline_fake_work, atomic_counter_ordering));
    });
    format::print_elapsed(true, "divide_parallel_settings_3", "", || {
        dbg!(divide_parallel_settings_3(low as u32, high as u32, min_split_size as u32, min_thread_size as u32, nsec_per_item as u8, inline_fake_work, use_atomic_counter));
    });
}

fn try_atomic_counter() {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let min_thread_size = 1_000;
    let nsec_per_item = 0;
    let inline_fake_work = true;
    let atomic_counter_ordering = None;
    format::print_elapsed(true, "divide_parallel_settings_2", "", || {
        dbg!(divide_parallel_settings_2(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, atomic_counter_ordering));
    });
    let atomic_counter_ordering = Some(atomic::Ordering::Relaxed);
    format::print_elapsed(true, "divide_parallel_settings_2", "", || {
        dbg!(divide_parallel_settings_2(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, atomic_counter_ordering));
    });
    dbg!(CALL_COUNTER.load(atomic::Ordering::Relaxed));
}

fn try_atomic_counter_orderings() {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let min_thread_size = 1_000;
    let nsec_per_item = 0;
    let inline_fake_work = true;
    for atomic_counter_ordering in [None, Some(atomic::Ordering::Relaxed), Some(atomic::Ordering::Release), Some(atomic::Ordering::Acquire), Some(atomic::Ordering::AcqRel), Some(atomic::Ordering::SeqCst)].iter() {
        CALL_COUNTER.store(0, atomic::Ordering::SeqCst);
        let ordering_label = match atomic_counter_ordering {
            Some(ordering) => format!("{:?}", ordering),
            None => "None".to_string(),
        };
        let case_label = format!("{}: ordering = {}", "divide_parallel_settings_2", &ordering_label);
        format::print_elapsed(true, &case_label, "", || {
            dbg!(divide_parallel_settings_2(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, *atomic_counter_ordering));
        });
        dbg!(CALL_COUNTER.load(atomic::Ordering::SeqCst));
    }
}

fn try_mutex_and_no_return() {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let min_thread_size = 1_000;
    let nsec_per_item = 0;
    let inline_fake_work = true;

    let atomic_counter_ordering = None;
    CALL_COUNTER.store(0, atomic::Ordering::SeqCst);
    format::print_elapsed(true, "divide_parallel_settings_2", "", || {
        dbg!(divide_parallel_settings_2(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, atomic_counter_ordering));
    });
    dbg!(CALL_COUNTER.load(atomic::Ordering::SeqCst));

    CALL_COUNTER.store(0, atomic::Ordering::SeqCst);
    THREAD_COUNTER.store(0, atomic::Ordering::SeqCst);
    let ordering = atomic::Ordering::AcqRel;
    format::print_elapsed(true, "divide_parallel_no_return", "", || {
        dbg!(divide_parallel_no_return(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work, ordering));
    });
    dbg!(CALL_COUNTER.load(atomic::Ordering::SeqCst));
    dbg!(THREAD_COUNTER.load(atomic::Ordering::SeqCst));

    format::print_elapsed(true, "divide_parallel_mutex", "", || {
        dbg!(divide_parallel_mutex(low, high, min_split_size, min_thread_size, nsec_per_item, inline_fake_work));
    });
}

struct MonitorCounts {
    calls: usize,
    threads: usize,
    live_threads: usize,
}

impl Debug for MonitorCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "calls: {:>7}\tthreads: {:>7}\tlive threads: {:>7}", format::format_count(self.calls), format::format_count(self.threads), format::format_count(self.live_threads))
    }
}

fn run_monitor(duration: time::Duration) -> Vec<MonitorCounts> {
    let mut v = vec![];
    while CALL_COUNTER.load(atomic::Ordering::SeqCst) == 0 {
        v.push(MonitorCounts {
            calls: 0,
            threads: 0,
            live_threads: 0,
        });
        thread::sleep(duration);
    }
    loop {
        let call_counter = CALL_COUNTER.load(atomic::Ordering::SeqCst);
        if call_counter == 0 {
            break;
        }
        v.push(MonitorCounts {
            calls: call_counter,
            threads: THREAD_COUNTER.load(atomic::Ordering::SeqCst),
            live_threads: LIVE_THREAD_COUNTER.load(atomic::Ordering::SeqCst),
        });
        thread::sleep(duration);
    }
    v
}

fn try_monitor() {
    let low = 0;
    let high = 1_000_000;
    let min_split_size = 10;
    let min_thread_size = 10_000;
    let nsec_per_item = 100;
    let ordering = atomic::Ordering::AcqRel;

    let duration = time::Duration::from_micros(1_000);

    CALL_COUNTER.store(0, atomic::Ordering::SeqCst);
    THREAD_COUNTER.store(0, atomic::Ordering::SeqCst);
    LIVE_THREAD_COUNTER.store(0, atomic::Ordering::SeqCst);
    let handle_work = thread::spawn(move || {
        divide_parallel_monitor(low, high, min_split_size, min_thread_size, nsec_per_item, ordering);
    });

    let handle_monitor = thread::spawn(move || run_monitor(duration));

    handle_work.join().unwrap();
    CALL_COUNTER.store(0, atomic::Ordering::SeqCst);
    THREAD_COUNTER.store(0, atomic::Ordering::SeqCst);
    LIVE_THREAD_COUNTER.store(0, atomic::Ordering::SeqCst);

    let monitor_counts = handle_monitor.join().unwrap();
    dbg!(&monitor_counts);

}

fn get_sizes() {
    dbg!(mem::size_of::<atomic::Ordering>());
    dbg!(mem::size_of::<DivideParallelSettings>());
    dbg!(mem::size_of::<DivideParallelSettings2>());
    dbg!(mem::size_of::<DivideParallelSettingsNoReturn>());
    dbg!(mem::size_of::<DivideParallelSettingsMonitor>());
    dbg!(mem::size_of::<DivideParallelSettingsMutex>());
    // dbg!(mem::size_of::<DivideParallelSettings2b>());
    // dbg!(mem::size_of::<DivideParallelSettings2c>());
    dbg!(mem::size_of::<DivideParallelSettings3>());
}

struct Counter {
    pub call_count: u32,
}

impl Counter {
    fn new() -> Self {
        Counter {
            call_count: 0,
        }
    }

    fn inc(&mut self, n: u32) {
        self.call_count += n;
    }
}


/*
    low: L,
    size: S,
    min_split_size: u8,
    min_thread_size: T,
    nsec_per_item: u8,
    ordering: atomic::atomic::Ordering,
*/

#[derive(Clone)]
struct CounterForGeneric {
    pub call_count: u32,
    pub settings_sizes: HashMap<u8, u32>,
    pub low_sizes: HashMap<u8, u32>,
    pub size_sizes: HashMap<u8, u32>,
}

impl CounterForGeneric {
    fn new() -> Self {
        Self {
            call_count: 0,
            settings_sizes: HashMap::new(),
            low_sizes: HashMap::new(),
            size_sizes: HashMap::new(),
        }
    }

    fn record(&mut self, call_count: u8, settings_size: u8, low_size: u8, size_size: u8) {
        self.call_count += call_count as u32;
        *(self.settings_sizes.entry(settings_size).or_insert(0)) += 1;
        *(self.low_sizes.entry(low_size).or_insert(0)) += 1;
        *(self.size_sizes.entry(size_size).or_insert(0)) += 1;
    }
}
