#![allow(unused_imports)]

#![feature(slice_partition_dedup)]
#![feature(is_sorted)]
#![feature(test)]
// #![unstable(feature = "unique", issue = "27730")]
#![feature(ptr_internals)]
#![feature(slice_partition_at_index)]
#![feature(drain_filter)]
// Needed for the time crate:
//#![feature(const_fn)]
//#![feature(rustc_const_unstable)]
// Needed for Rocket (http::http_server).
#![feature(proc_macro_hygiene, decl_macro)]
#![feature(map_first_last)]
#![feature(convert_float_to_int)]
#![feature(binary_heap_into_iter_sorted)]

#[macro_use]
extern crate util;
pub use util::*;

extern crate voronoi;
extern crate cogset;
extern crate polygon2;
#[macro_use] extern crate geo;
extern crate geo_booleanop;

extern crate rand;
pub use rand::*;

#[macro_use] extern crate lazy_static;

extern crate num_format;

extern crate itertools;

extern crate ordered_float;

extern crate test;

#[macro_use] extern crate rocket;
// extern crate rocket_cors;
extern crate serde;
extern crate serde_json;


// extern crate time;

pub mod coord;
pub mod counter;
pub mod http;
pub mod map;
// pub mod polygon_map;
pub mod range;
pub mod sort;
pub mod vis;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
