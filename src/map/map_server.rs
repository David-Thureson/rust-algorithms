#![allow(dead_code)]

use std::sync::Mutex;
use rocket_contrib;
use rocket::response::content;
use crate::http::http_server;
use serde::Serialize;
use serde_json;

use super::*;

lazy_static! {
    pub static ref MAP_OPTIONS: Mutex<Option<D3MapOptions>> = Mutex::new(None);
}

#[derive(Debug, Clone, Copy)]
pub enum D3LabelLevel {
    None,
    Sequence,
    RelatedSequences,
    Altitude,
}

#[derive(Debug, Clone, Copy)]
pub struct D3MapOptions {
    pub draw_points: bool,
    pub draw_polygon_areas: bool,
    pub draw_polygon_lines: bool,
    pub draw_edges: bool,
    pub polygon_label_level: D3LabelLevel,
    pub vertex_label_level: D3LabelLevel,
    pub edge_label_level: D3LabelLevel,
    pub terrain_opacity: f32,
    pub ocean_guide_opacity: f32,
}

impl D3MapOptions {
    pub fn for_points() -> Self {
        Self {
            draw_points: true,
            draw_polygon_areas: false,
            draw_polygon_lines: false,
            draw_edges: false,
            polygon_label_level: D3LabelLevel::None,
            vertex_label_level: D3LabelLevel::None,
            edge_label_level: D3LabelLevel::None,
            terrain_opacity: 1.0,
            ocean_guide_opacity: 0.0,
        }
    }

    pub fn for_polygons() -> Self {
        Self {
            draw_points: true,
            draw_polygon_areas: true,
            draw_polygon_lines: true,
            draw_edges: false,
            polygon_label_level: D3LabelLevel::None,
            vertex_label_level: D3LabelLevel::None,
            edge_label_level: D3LabelLevel::None,
            terrain_opacity: 1.0,
            ocean_guide_opacity: 0.0,
        }
    }

    pub fn for_vertices_and_edges(label_level: D3LabelLevel) -> Self {
        Self {
            draw_points: false,
            draw_polygon_areas: true,
            draw_polygon_lines: false,
            draw_edges: true,
            polygon_label_level: label_level,
            vertex_label_level: label_level,
            edge_label_level: label_level,
            terrain_opacity: 1.0,
            ocean_guide_opacity: 0.0,
        }
    }

    pub fn for_ocean_guide() -> Self {
        Self {
            draw_points: false,
            draw_polygon_areas: true,
            draw_polygon_lines: false,
            draw_edges: false,
            polygon_label_level: D3LabelLevel::None,
            vertex_label_level: D3LabelLevel::None,
            edge_label_level: D3LabelLevel::None,
            terrain_opacity: 0.9,
            ocean_guide_opacity: 0.1,
        }
    }

    pub fn for_water() -> Self {
        Self {
            draw_points: false,
            draw_polygon_areas: true,
            draw_polygon_lines: false,
            draw_edges: false,
            polygon_label_level: D3LabelLevel::None,
            vertex_label_level: D3LabelLevel::None,
            edge_label_level: D3LabelLevel::None,
            terrain_opacity: 1.0,
            ocean_guide_opacity: 0.0,
        }
    }

    fn opacity(&self, layer_type: LayerType) -> f32 {
        match layer_type {
            LayerType::Terrain => self.terrain_opacity,
            LayerType::OceanGuide => self.ocean_guide_opacity,
        }
    }

    pub fn adjust_color_for_layer(&self, color: &D3Color, layer_type: LayerType) -> D3Color {
        let opacity = self.opacity(layer_type);
        color.mult_a(opacity)
    }

    pub fn adjust_edge_opacity_for_labels(&self, edge_color: &D3Color) -> D3Color {
        match (self.vertex_label_level, self.edge_label_level) {
            (D3LabelLevel::None, D3LabelLevel::None) => { edge_color.clone() },
            _ => { edge_color.mult_a(0.5) }
        }
    }
}

#[get("/polygon_map_anim_1?<point_count>&<polygon_step>")]
fn polygon_map_anim_1(point_count: usize, polygon_step: usize) -> content::Json<String> {
    let mut map = polygon_map::gen_map(point_count);
    if polygon_step > 0 {
        map.add_voronoi_polygons();
        for _ in 2..=polygon_step {
            map.terrain.relax_polygons();
        }
    }
    send_map(map, D3MapOptions::for_polygons())
}

#[get("/polygon_map_gen_points?<point_count>")]
#[allow(unused_variables)]
fn polygon_map_gen_points(point_count: usize) -> content::Json<String> {
    // let map = polygon_map::gen_map(point_count);
    let map = try_ocean_multi_level(point_count);

    // send_map(map, D3MapOptions::for_points())
    send_map(map, D3MapOptions::for_water())
}

#[get("/polygon_map_gen_polygons")]
fn polygon_map_gen_polygons() -> content::Json<String> {
    let mut map= map_take().unwrap();
    map.terrain.add_voronoi_polygons();
    send_map(map, D3MapOptions::for_polygons())
}

// Experiment with holding on to the lock while working on the map.
/*
#[get("/polygon_map_gen_polygons")]
fn polygon_map_gen_polygons() -> content::Json<String> {
    let mut guard: MutexGuard<Option<polygon_map::PolygonMap>> = polygon_map::MAP.lock().unwrap();
    let map: &mut polygon_map::PolygonMap = guard.as_mut().unwrap();
    map.frames[0].add_voronoi_polygons();
    send_map_alternative(map)
}
*/

#[get("/polygon_map_relax_polygons")]
fn polygon_map_relax_polygons() -> content::Json<String> {
    let mut map= map_take().unwrap();
    map.terrain.relax_polygons();
    send_map(map, D3MapOptions::for_polygons())
}

#[get("/polygon_map_gen_vertices_and_edges")]
fn polygon_map_gen_vertices_and_edges() -> content::Json<String> {
    let mut map= map_take().unwrap();
    map.terrain.gen_vertices_and_edges();
    let label_level = match map.terrain.points.len() {
        0 ... 9 => D3LabelLevel::RelatedSequences,
        10 ... 50 => D3LabelLevel::Sequence,
        _ => D3LabelLevel::None,
    };
    send_map(map, D3MapOptions::for_vertices_and_edges(label_level))
}

#[get("/polygon_map_gen_ocean_guide?<ocean_point_count>&<ocean_relax_level>&<ocean_pct>")]
fn polygon_map_gen_ocean_guide(ocean_point_count: usize, ocean_relax_level: u8, ocean_pct: u8) -> content::Json<String> {

    let mut map= map_take().unwrap();

    let mut terrain_goal = map.terrain.layer_state.clone();
    terrain_goal.water = true;

    let mut ocean_guide_goal = LayerState::new();
    ocean_guide_goal.point_count = ocean_point_count;
    ocean_guide_goal.polygons = true;
    ocean_guide_goal.relax_level = ocean_relax_level;
    ocean_guide_goal.water_pct = f(ocean_pct) / f(100.0);

    map.take_to_goal(&terrain_goal, Some(&ocean_guide_goal), false, false);

    let map = try_ocean_multi_level(1000);

    send_map(map, D3MapOptions::for_ocean_guide())
}

/*
#[get("/polygon_map_fix_polygons")]
fn polygon_map_fix_polygons() -> content::Json<String> {
    let mut map= map_take().unwrap();
    map.frames[0].fix_polygons();
    send_map(map)
}
*/

#[get("/polygon_map_rotate_map")]
fn polygon_map_rotate_map() -> content::Json<String> {
    let mut map= map_take().unwrap();
    map.rotate();
    // Keep whatever drawing options we used last time.
    send_map(map, map_options_take().unwrap())
}

#[get("/polygon_map_call_time")]
fn polygon_map_call_time() -> content::Json<String> {
    let a = "abc";
    content::Json(serde_json::to_string(&a).unwrap())
}

fn send_map(map: polygon_map::PolygonMap, options: D3MapOptions) -> content::Json<String> {
    let d3_map = map.gen_d3(options.clone());

    // dbg!(&d3_map, &options);

    let content = content::Json(serde_json::to_string(&d3_map).unwrap());
    let content = serde_json::to_string(&d3_map);
    map_replace(map);
    map_options_replace(options);
    content
}

/*
fn send_map_alternative(map: &polygon_map::PolygonMap) -> content::Json<String> {
    let d3_map = map.gen_d3();
    let content = content::Json(serde_json::to_string(&d3_map).unwrap());
    content
}
*/

/*
#[get("/polygon_map_1")]
fn polygon_map_1() -> content::Json<String> {
    let map = gen_d3_map(polygon_map::POINT_COUNT);
    /*
    let map = PolygonMap {
        points: vec![Point{ key: "a".to_string(), x: 3.0, y: 5.8}, Point { key: "b".to_string(), x:19.5, y: 45.7 }],
    };
    */
    content::Json(serde_json::to_string(&map).unwrap())
}
*/
/*
#[get("/polygon_map_1")]
fn polygon_map_1() -> rocket_contrib::json::Json<D3Map> {
    let map = gen_d3_map(polygon_map::POINT_COUNT);
    /*
    let map = PolygonMap {
        points: vec![Point{ key: "a".to_string(), x: 3.0, y: 5.8}, Point { key: "b".to_string(), x:19.5, y: 45.7 }],
    };
    */
    rocket_contrib::json::Json(map)
}
*/

pub fn map_options_take() -> Option<D3MapOptions> {
    MAP_OPTIONS.lock().unwrap().take()
}

pub fn map_options_replace(map_options: D3MapOptions) -> Option<D3MapOptions> {
    MAP_OPTIONS.lock().unwrap().replace(map_options)
}

pub fn send_to_d3() {
    http_server::start(routes![
        polygon_map_gen_points, polygon_map_gen_polygons,
        polygon_map_relax_polygons, polygon_map_gen_vertices_and_edges, polygon_map_gen_ocean_guide,
        polygon_map_rotate_map, polygon_map_call_time, polygon_map_anim_1
    ]).unwrap();
}

