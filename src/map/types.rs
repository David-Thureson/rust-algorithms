use ordered_float::NotNaN;
use std::sync::{atomic, Mutex};
use std::ops::{Mul, Deref};
use std::convert::FloatToInt;

use crate::map::MAP_WIDTH;
use util::format;

static NEXT_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

pub const PCT_100: Pct = Pct { 0: 1.0 };
pub const PCT_75: Pct = Pct { 0: 0.75 };
pub const PCT_50: Pct = Pct { 0: 0.5 };
pub const PCT_25: Pct = Pct { 0: 0.25 };
pub const PCT_0: Pct = Pct { 0: 0.0 };

pub type F = NotNaN<f64>;

#[derive(Clone, Copy)]
pub struct Pct(f64);

// UD3 and ID3 are used for the objects sent to the web page for use in D3.js code.
pub type UD3 = u16;
pub type ID3 = i16;

#[derive(Clone)]
pub struct Bounds {
    pub x_min: F,
    pub x_max: F,
    pub y_min: F,
    pub y_max: F,
    pub scale_to_d3: Option<F>,
}


pub fn f<T> (val: T) -> F
    where T: Into<f64>
{
    let val: f64 = val.into();
    val.into()
}

pub fn pct<T> (val: T) -> Pct
    where T: Into<f64>
{
    let val: f64 = val.into();
    assert!(val >= 0.0);
    assert!(val <= 1.0);
    val.into()
}

/*
pub fn f<T> (val: T) -> F
    where T: Into<f64>
{
    F::from(val as f64)
}
*/

pub fn xy_to_d3(x: F, y: F) -> (ID3, ID3) {
    (f_to_d3(x), f_to_d3(y))
}

/*
pub fn x_y_to_d3(x: F, y: F, bounds: &Bounds) -> (ID3, ID3) {
    (
        x_to_d3(x, bounds),
        y_to_d3(y, bounds),
    )
}
*/

/*
pub fn x_to_d3(x: F, bounds: &Bounds) -> ID3 {
    (*(((x - bounds.x_min) * bounds.scale_to_d3.unwrap()) + f(MAP_H_PADDING))) as ID3
}
*/

pub fn f_to_d3(val: F) -> ID3 {
    (*(val * f(MAP_WIDTH))) as ID3
}

pub fn f_mean<'a>(val_iter: impl Iterator<Item = &'a F>) -> F {
    let (count, sum) = val_iter.fold((0, 0.0.into()), |acc: (usize, F), x| (acc.0 + 1, acc.1 + *x));
    let count = f(count as f64);
    sum / count
}

pub fn format_f(val: F) -> String {
    format!("{:.4}", *val)
}

pub fn format_f_opt(val: Option<F>) -> String {
    if let Some(val) = val {
        format_f(val)
    } else {
        "?".to_string()
    }
}

pub fn format_f_labeled(val: F, label: &str) -> String {
    format!(", {}: {}", label, format_f(val))
}

pub fn format_f_opt_labeled(val: Option<F>, label: &str) -> String {
    if let Some(val) = val { format_f_labeled(val, label) } else { "".to_string() }
}

pub fn format_count_labeled(val: usize, label: &str) -> String {
    format!(", {}: {}", label, format::format_count(val))
}

pub fn format_count_opt_labeled(val: Option<usize>, label: &str) -> String {
    if let Some(val) = val { format_count_labeled(val, label) } else { "".to_string() }
}

pub fn id() -> usize {
    NEXT_ID.fetch_add(1, atomic::Ordering::AcqRel)
}

/*
impl <T> Into<T> for F
    where T: Into<f64>
{
    fn into(self) -> F {
        F{ 0: self as f64 }
    }
}
*/

impl Mul<Pct> for F {
    type Output = F;

    fn mul(self, rhs: Pct) -> Self::Output {
        self * f(*rhs)
    }
}

/*
impl <T> Mul<T> for Pct
    where T: Into<f64>,
        f64: FloatToInt<T>
{
    type Output = T;

    fn mul(self, rhs: T) -> Self::Output {
        (*self * (rhs as f64)).round_unchecked_to()
    }
}
*/

impl Mul<usize> for Pct {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        (*self * (rhs as f64)) as usize
    }
}

impl Deref for Pct {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/*
impl Into<F> for Pct {
    fn into(self) -> F {
        f(*self)
    }
}
*/

/*
impl <T> Into<T> for Pct
    where T: Into<f64>
{
    fn into(self) -> Pct {
        let val = self as f64;
        assert!(val >= 0.0);
        assert!(val <= 1.0);
        Pct { 0: val }
    }
}
*/

impl <T> From<T> for Pct
    where f64: From<T>
{
    fn from(val: T) -> Self {
        let val = f64::from(val);
        assert!(val >= 0.0);
        assert!(val <= 1.0);
        Pct { 0: val }
    }
}

impl Bounds {
    pub fn new(x_min: F, x_max: F, y_min: F, y_max: F) -> Self {
        Self { x_min, x_max, y_min, y_max, scale_to_d3: None }
    }

    pub fn width(&self) -> F {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> F {
        self.y_max - self.y_min
    }

}

