use std::fmt::{self, Debug};

use super::*;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Point {
    // Keep x and y first because we're deriving Ord and PartialOrd and want those to be based on
    // x and y.
    pub x: F,
    pub y: F,
    pub id: usize,
}

impl Point {
    pub fn new(x: F, y: F) -> Self {
        Self { id: id(), x, y }
    }

    pub fn gen_d3(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        if options.draw_points {
            let (cx, cy) = self.xy_to_d3();
            d3_map.circles.push(D3Circle::new(&self.id.to_string(), cx, cy, VOR_POINT_R, &VOR_POINT_FILL));
        }
    }

    pub fn xy_to_d3(&self) -> (ID3, ID3) {
        xy_to_d3(self.x, self.y)
    }

    // pub fn x_y_to_d3(&self, bounds: &Bounds) -> (ID3, ID3) {
    //     x_y_to_d3(self.x, self.y, bounds)
    // }

    pub fn to_voronoi_point(&self) -> voronoi::Point {
        voronoi::Point::new(self.x.into(), self.y.into())
    }

    pub fn from_voronoi_point(p: &voronoi::Point) -> Self {
        Self::new((*p.x).into(), (*p.y).into())
    }

    pub fn update_from_voronoi_point(&mut self, p: &voronoi::Point) {
        self.x = (*p.x).into();
        self.y = (*p.y).into();
    }

    pub fn rotate(&mut self, axis_point: &Point) {
        let mut p = self.sub(axis_point);
        let (x, y) = (p.x, p.y);
        p.x = -y;
        p.y = x;
        let p = p.add(axis_point);
        self.x = p.x;
        self.y = p.y;
    }

    pub fn add(&self, other: &Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(&self, other: &Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }

    pub fn key(&self) -> String {
        format!("{}:{}", (*self.x) as f32, (*self.y) as f32)
    }

    pub fn pair_key(mut points: [&Point; 2]) -> String {
        points.sort();
        format!("{},{}", points[0].key(), points[1].key())
    }

    pub fn point_bounds(points: &[&Point]) -> (F, F, F, F) {
        let x_min = points.iter().map(|p| p.x).min().unwrap();
        let x_max = points.iter().map(|p| p.x).max().unwrap();
        let y_min = points.iter().map(|p| p.y).min().unwrap();
        let y_max = points.iter().map(|p| p.y).max().unwrap();
        (x_min, x_max, y_min, y_max)
    }

    pub fn mean<'a>(points_iter: impl Iterator<Item = &'a Point>) -> Point {
        let (count, x_sum, y_sum) = points_iter.fold((0, 0.0.into(), 0.0.into()), |acc: (usize, F, F), p| (acc.0 + 1, acc.1 + p.x, acc.2 + p.y));
        let count = f(count as f64);
        Point::new(x_sum / count, y_sum / count)
    }

    pub fn to_polygon2_array(&self) -> [f64; 2] {
        [*self.x, *self.y]
    }

    pub fn distance_to(&self, other: &Point) -> F {
        let x = (self.x - other.x).abs();
        let y = (self.y - other.y).abs();
        ((x * x) + (y * y)).sqrt().into()
    }
    /*
    pub fn mean(points: &[&Point]) -> Point {
        debug_assert!(points.len() > 0);
        let count: F = f(points.len() as f64);
        let x_sum: F = points.iter().map(|p| p.x).fold(0.0.into(), |acc: F, x| acc + x);
        let y_sum: F = points.iter().map(|p| p.y).fold(0.0.into(), |acc: F, y| acc + y);
        Point::new(x_sum / count, y_sum / count)
    }
    */

    /*
    pub fn as_bounds(points: &[&Point]) -> Bounds {
        let (x_min, x_max, y_min, y_max) = Point::point_bounds(points);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
            scale_to_d3: None,
        }
    }
    */

}

impl Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point {{ id: {}, x: {}, y: {} }}", self.id, format_f(self.x), format_f(self.y))
    }
}

/*
pub trait HasPoints {
    fn point_iter() -> Iter;

    fn point_mean() -> Point {

    }
}
*/