// Inspired by Polygonal Map Generation for Games: http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/

#![allow(dead_code)]

use std::fmt::{self, Debug};
use std::collections::{BTreeSet, BTreeMap, HashSet, BinaryHeap};
use rand::prelude::ThreadRng;
use rand::Rng;
use std::sync::{atomic, Mutex, MutexGuard};
use crate::voronoi;
use crate::cogset::{Euclid, Kmeans};
use rocket_contrib;
use serde_json;
// use polygon2;
// use geo::prelude;
// use geo::Polygon;
// use geo::{line_string};
// use geo::convexhull::ConvexHull;
// use geo_booleanop::boolean::BooleanOp;

use std::cmp::{min, Ordering};
use std::convert::TryFrom;
use std::borrow::BorrowMut;
use std::ops::{Sub, Add, AddAssign};
use crate::itertools::Itertools;
use std::collections::HashMap;
use ordered_float::NotNaN;

use super::*;

static NEXT_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

lazy_static! {
    pub static ref MAP_MUTEX: Mutex<usize> = Mutex::new(0);
}

pub static MAP: Option<PolygonMap> = None;

pub const POINT_COUNT: usize = 200;

pub const SHOW_SEQUENCE_LABELS: bool = true;

pub const MAP_WIDTH: UD3 = 800;
pub const MAP_HEIGHT: UD3 = 800;
pub const MAP_H_PADDING: UD3 = 0;
pub const MAP_V_PADDING: UD3 = 0;
pub const VOR_POINT_R: UD3 = 1;
pub const VOR_POINT_FILL: D3Color = COLOR_GREEN;
pub const EDGE_STROKE_WIDTH: UD3 = 1;
pub const TERRAIN_EDGE_STROKE: D3Color = COLOR_GRAY.a(0.5);
pub const OCEAN_GUIDE_EDGE_STROKE: D3Color = COLOR_GREEN.a(0.5);
pub const DEBUG_LABEL_FONT_FAMILY: D3FontFamily = D3FontFamily::SansSerif;
pub const DEBUG_LABEL_FONT_SIZE: UD3 = 10;
pub const DEBUG_LABEL_FILL: D3Color = COLOR_BLACK;
pub const DEBUG_LABEL_TEXT_ANCHOR: D3TextAnchor = D3TextAnchor::Middle;
pub const POLYGON_UNKNOWN_FILL: D3Color = COLOR_LIGHT_GRAY;
pub const POLYGON_BLANK_FILL: D3Color = COLOR_CLEAR;
pub const POLYGON_LAND_FILL: D3Color = COLOR_WHITE;
pub const POLYGON_OCEAN_FILL: D3Color = COLOR_BLUE;
pub const POLYGON_OCEAN_GUIDE_FILL: D3Color = COLOR_GREEN;

pub fn main() {
    // try_voronoi();
    // try_gen_d3();
    // try_cogset();
    // try_fix_polygons();
    // try_rotate();
    // try_gen_vertices_and_edges();
    try_distance_to_water();
    map_server::send_to_d3();
}

pub type F = NotNaN<f64>;

/*
pub struct F(f64);

impl Into<F> for UD3 {
    fn into(self) -> F {
        F { 0: self as f64 }
    }
}

impl Into<F> for f64 {
    fn into(self) -> F {
        assert!(!self.is_normal());
        F { 0: self }
    }
}
*/

#[derive(Debug, Clone)]
pub struct PolygonMap {
    pub id: usize,
    pub terrain: Layer,
    pub ocean_guide: Layer,
}

#[derive(Clone)]
pub struct LineSegment {
    pub id: usize,
    pub points: [Point; 2],
}

#[derive(Clone)]
pub struct Bounds {
    pub x_min: F,
    pub x_max: F,
    pub y_min: F,
    pub y_max: F,
    pub scale_to_d3: Option<F>,
}

impl PolygonMap {
    fn new() -> Self {
        Self {
            id: id(),
            terrain: Layer::new(LayerType::Terrain),
            ocean_guide: Layer::new(LayerType::OceanGuide),
        }
    }

    pub fn add_voronoi_polygons(&mut self) {
        self.terrain.add_voronoi_polygons();
    }

    pub fn take_to_goal(&mut self, terrain_goal_state: &LayerState, ocean_guide_goal_state: Option<&LayerState>, terrain_force_new: bool, ocean_guide_force_new: bool) {
        self.terrain.take_to_pre_water_state(terrain_goal_state, terrain_force_new);
        if let Some(ocean_guide_goal_state) = ocean_guide_goal_state {
            let mut ocean_guide_changed = self.ocean_guide.take_to_pre_water_state(ocean_guide_goal_state, ocean_guide_force_new);
            if ocean_guide_goal_state.water_pct != self.ocean_guide.layer_state.water_pct {
                self.ocean_guide.set_water_from_pct(ocean_guide_goal_state.water_pct);
                ocean_guide_changed = true;
            }
            if ocean_guide_changed {
                self.terrain.clear_water();
            }
            if terrain_goal_state.water && !self.terrain.layer_state.water {
                //self.set_terrain_water_from_ocean_guide();
                // self.set_terrain_water_multi_level(ocean_guide_goal_state);
            }
        }
    }

    pub fn set_terrain_water_from_ocean_guide(guide_layer: &Layer, target_layer: &mut Layer) {
        let mut rng = rand::thread_rng();
        for polygon in target_layer.polygons.iter_mut() {
            polygon.polygon_type = PolygonType::Blank;
        }
        for ocean_guide_polygon in guide_layer.polygons.iter() {
            match ocean_guide_polygon.polygon_type {
                PolygonType::Ocean => {
                    let ocean_guide_polygon_points = ocean_guide_polygon.to_polygon2_arrays();
                    for polygon in target_layer.polygons.iter_mut() {
                        if Polygon::contains_enough_points(&ocean_guide_polygon_points, &polygon.to_polygon2_arrays(), &mut rng) {
                            polygon.polygon_type = PolygonType::Ocean;
                        };
                    }
                },
                _ => {},
            }
        }
    }

    pub fn gen_d3(&self, mut options: D3MapOptions) -> D3Map {
        let mut d3_map = D3Map::new(MAP_WIDTH, MAP_HEIGHT, MAP_H_PADDING, MAP_V_PADDING);
        self.terrain.gen_d3(&mut d3_map, &mut options);
        self.ocean_guide.gen_d3(&mut d3_map, &mut options);
        d3_map
    }

    pub fn rotate(&mut self) {
        let axis_point = Point::new(0.5.into(), 0.5.into());
        self.terrain.rotate(&axis_point);
        self.ocean_guide.rotate(&axis_point);
    }

}

/*

// This can be used for polygons, vertices, and edges, all of which may be indexed by usize.
struct ItemDistance {
    dist: F,
    index: usize,
}

impl ItemDistance {
    fn new(dist: F, index: usize) -> Self {
        Self { dist, index }
    }
}

impl Ord for ItemDistance {
    fn cmp(&self, other: &ItemDistance) -> Ordering {
        // Flip the usual order and start with _other_ in the next line because we want to get the
        // item with the lowest distance first.
        other.dist.cmp(&self.dist)
    }
}

impl PartialOrd for ItemDistance {
    fn partial_cmp(&self, other: &ItemDistance) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct VertexDistanceIterator<'a> {
    map: &'a PolygonMap,
    start_point: Point,
    vertex_queue: BinaryHeap<ItemDistance>,
    noted_vertices: HashSet<usize>,
}

impl VertexDistanceIterator {
    fn new(vertex: &Vertex) -> Self {
        let mut iter = VertexDistanceIterator {
            start_point: vertex.point.clone(),
            vertex_queue: BinaryHeap::new(),
            noted_vertices: HashSet::new(),
        };
        iter.vertex_queue.push(Item);
        iter.noted_vertices.insert(vertex.sequence);
        iter
    }
}

impl Iterator for VertexDistanceIterator {
    type Item = Vertex;

    fn next(&mut self) -> Option<Self::Item> {
        // Pull the closest vertex from the queue (which at first will be the starting vertex
        // itself) and add to the queue all of the vertices that this vertex touches and that we
        // haven't seen before.
        if let Some(item_distance) = self.vertex_queue.pop() {

        }
    }
}
*/


impl LineSegment {
    fn new(point_1: &Point, point_2: &Point) -> Self {
        Self { id: id(), points: [point_1.clone(), point_2.clone()] }
    }

    fn y_intercept(&self) -> Option<F> {
        let LineSegment { id: _, points: [ p0, p1 ] } = self.in_x_order();
        if p0.x == p1.x || p0.x > 0.0.into() || p1.x < 0.0.into() {
            return None;
        }
        let x_proportion = p0.x / (p1.x - p0.x);
        let y_length = p1.y - p0.y;
        let y_to_intercept = y_length * x_proportion;
        Some(p0.y + y_to_intercept)
    }

    fn in_x_order(&self) -> Self {
        if self.points[0].x < self.points[2].x {
            Self { id: self.id, points: [self.points[0].clone(), self.points[1].clone()] }
        } else {
            Self { id: self.id, points: [self.points[1].clone(), self.points[0].clone()] }
        }
    }

    fn midpoint(&self) -> Point {
        Point::mean(self.points.iter())
    }
}

impl Bounds {
    pub fn new(x_min: F, x_max: F, y_min: F, y_max: F) -> Self {
        Self { x_min, x_max, y_min, y_max, scale_to_d3: None }
    }

    pub fn from_points(points: &[&Point]) -> Self {
        let (x_min, x_max, y_min, y_max) = Point::point_bounds(points);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
            scale_to_d3: None,
        }
    }

    pub fn width(&self) -> F {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> F {
        self.y_max - self.y_min
    }

}

pub fn gen_map(point_count: usize) -> PolygonMap {
    let mut rng = rand::thread_rng();
    let mut map = PolygonMap::new();
    for _ in 0..point_count {
        map.terrain.points.push(Point::new(
            rng.gen::<f64>().into(),
            rng.gen::<f64>().into(),
            //0.5 + (rng.gen::<f64>() / 0.5),
            //0.5 + (rng.gen::<f64>() / 0.5)
        ));
    }
    // frame.add_voronoi_polygons();
    map
}

pub fn map_take() -> Option<PolygonMap> {
    MAP.lock().unwrap().take()
}

pub fn map_get_clone() -> Option<PolygonMap> {
    MAP.lock().unwrap().clone()
}

pub fn map_replace(map: PolygonMap) -> Option<PolygonMap> {
    MAP.lock().unwrap().replace(map)
}

pub fn map_replace_clone(map: &PolygonMap) {
    MAP.lock().unwrap().replace(map.clone());
}

// pub fn map_get_guard() -> MutexGuard<Option<PolygonMap>> + 'static {
//    MAP.lock().unwrap()
// }

/*
pub fn try_hold_map_lock() {
    let mut guard: MutexGuard<Option<PolygonMap>> = MAP.lock().unwrap();
    let map = guard.as_mut().unwrap();
    map.make_changes();
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

pub fn x_to_d3(x: F, bounds: &Bounds) -> ID3 {
    //bg!(x, bounds);
    //bg!(x - bounds.x_min);
    //bg!((x - bounds.x_min) * bounds.scale_to_d3.unwrap());
    (*(((x - bounds.x_min) * bounds.scale_to_d3.unwrap()) + f(MAP_H_PADDING))) as ID3
}

pub fn y_to_d3(y: F, bounds: &Bounds) -> ID3 {
    (*(((y - bounds.y_min) * bounds.scale_to_d3.unwrap()) + f(MAP_V_PADDING))) as ID3
}

pub fn f_to_d3(val: F) -> ID3 {
    (*(val * f(MAP_WIDTH))) as ID3
}

/*
fn try_gen_d3() {
    let map = gen_map(10);
    dbg!(&map);
    let d3 = map.gen_d3();
    dbg!(&d3);
    let json = rocket_contrib::json::Json(&d3);
    dbg!(&json);
    let a  = serde_json::to_string_pretty(&d3).unwrap();
    dbg!(&a);
    let a  = serde_json::to_string(&d3).unwrap();
    dbg!(&a);
}
*/

/*
fn try_voronoi() {
    let map = gen_map(2);
    const BOX_SIZE: f64 = 1.0;
    let vor_pts = map.terrain.points.iter().map(|p| voronoi::Point::new(p.x.into(), p.y.into())).collect();
    let vor_diagram = voronoi::voronoi(vor_pts, BOX_SIZE);
    let vor_polys = voronoi::make_polygons(&vor_diagram);
    dbg!(&map.terrain.points);
    dbg!(&vor_polys);
}

fn try_cogset() {
    let map = gen_map(5);
    dbg!(&map);
    let data: Vec<Euclid<[f64; 2]>> = map.terrain.vor_points.iter().map(|point| Euclid([point.x, point.y])).collect();
    let k = 2;
    let kmeans = Kmeans::new(&data, k);
    dbg!(&kmeans.clusters());
}

fn try_fix_polygons() {
    let mut map = gen_map(1000);
    map.terrain.add_voronoi_polygons();
    map.terrain.fix_polygons();
}
*/

pub fn f<T> (val: T) -> F
    where T: Into<f64>
{
    let val: f64 = val.into();
    val.into()
}

pub fn f_mean<'a>(val_iter: impl Iterator<Item = &'a F>) -> F {
    let (count, sum) = val_iter.fold((0, 0.0.into()), |acc: (usize, F), x| (acc.0 + 1, acc.1 + *x));
    let count = f(count as f64);
    sum / count
}

pub fn format_f(val: F) -> String {
    format!("{:.4}", val)
}

pub fn format_f_opt(val: Option<F>) -> String {
    if let Some(val) = val {
        format_f(val)
    } else {
        "?".to_string()
    }
}

pub fn format_f_labeled(val: Option<F>, label: &str) -> String {
    if let Some(val) = val { format!(", {}: {}", label, format_f(val)) } else { "".to_string() }
}

pub fn id() -> usize {
    NEXT_ID.fetch_add(1, atomic::Ordering::AcqRel)
}

fn try_gen_vertices_and_edges() {
    let mut map = gen_map(10000);
    map.terrain.add_voronoi_polygons();
    map.terrain.gen_vertices_and_edges();
    map.terrain.relax_polygons();
    map.terrain.gen_vertices_and_edges();
}

pub fn try_ocean_multi_level() -> PolygonMap {
    // let terrain_point_count = 1000;
    let terrain_point_count = 100;
    let terrain_relax_level = 2;
    let first_ocean_point_count = 20;
    let ocean_pct = f(0.40);
    //let ocean_point_count_mult = 10;
    let ocean_point_count_mult = 2;
    let ocean_relax_level = 0;

    let mut terrain_goal = LayerState::new();
    terrain_goal.point_count = terrain_point_count;
    terrain_goal.polygons = true;
    terrain_goal.relax_level = terrain_relax_level;

    let mut map = PolygonMap::new();
    map.terrain.take_to_pre_water_state(&terrain_goal, true);

    let mut ocean_point_count = first_ocean_point_count;
    let mut prev_layer: Option<Layer> = None;
    while ocean_point_count < terrain_point_count {
        let mut ocean_goal = LayerState::new();
        ocean_goal.point_count = ocean_point_count;
        ocean_goal.polygons = true;
        ocean_goal.relax_level = ocean_relax_level;
        let mut this_layer = Layer::new(LayerType::OceanGuide);
        this_layer.take_to_pre_water_state(&ocean_goal, true);

        if let Some(guide_layer) = prev_layer.take() {
            PolygonMap::set_terrain_water_from_ocean_guide(&guide_layer, &mut this_layer);
        } else {
            this_layer.set_water_from_pct(ocean_pct);
        }
        prev_layer = Some(this_layer);
        ocean_point_count *= ocean_point_count_mult;
    }

    PolygonMap::set_terrain_water_from_ocean_guide(&prev_layer.unwrap(), &mut map.terrain);

    map
}

pub fn try_distance_to_water() {
    let mut map = try_ocean_multi_level();
    map.terrain.gen_vertices_and_edges();
    map.terrain.calc_distance_to_water();

}
