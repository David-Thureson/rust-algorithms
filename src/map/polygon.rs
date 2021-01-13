use std::fmt::{self, Debug};
use rand::prelude::ThreadRng;
use rand::Rng;
use itertools::Itertools;

use super::*;

#[derive(Debug, Clone, Eq)]
pub enum PolygonType {
    Unknown,
    Blank,
    Land,
    Ocean,
}

#[derive(Clone)]
pub struct Polygon {
    pub id: usize,
    pub sequence: usize,
    pub polygon_type: PolygonType,
    pub layer_type: LayerType,
    pub points: Vec<Point>,
    pub vertex_indexes: Vec<usize>,
    pub edge_indexes: Vec<usize>,
    pub seed_point: Option<Point>,
    pub altitude: Option<F>,
    pub area: Option<F>,
}

impl PartialEq for PolygonType {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Polygon {

    pub fn new(sequence: usize, polygon_type: PolygonType, layer_type: LayerType, seed_point: Option<Point>) -> Self {
        Self {
            id: id(),
            sequence,
            polygon_type,
            layer_type,
            points: vec![],
            vertex_indexes: vec![],
            edge_indexes: vec![],
            seed_point,
            altitude: None,
            area: None,
        }
    }

    pub fn rotate(&mut self, axis_point: &Point) {
        for point in self.points.iter_mut() {
            point.rotate(axis_point);
        }
        if let Some(seed_point) = self.seed_point.as_mut() {
            seed_point.rotate(axis_point);
        }
    }

    pub fn x_min(&self) -> F {
        self.points.iter().map(|p| p.x).min().unwrap()
    }

    pub fn calc_area(&mut self) {
        let points: Vec<[f64;2]> = self.points
            .iter()
            .map(|point| [*point.x, *point.y])
            .collect();
        let a: f64 = polygon2::area(&points);
        self.area = Some(a.into());
    }

    pub fn midpoint(&self) -> Point {
        Point::mean(self.points.iter())
    }

    pub fn point_pairs(&self) -> Vec<[(usize, &Point); 2]> {
        let mut v = vec![];
        for point_index in 0..self.points.len() {
            let point_index_next = if point_index == self.points.len() { 0 } else { point_index + 1 };
            v.push([
                (point_index, &self.points[point_index]),
                (point_index_next, &self.points[point_index_next]),
            ]);
        }
        v
    }

    pub fn vertex_index_ordered_pairs(&self) -> Vec<[usize; 2]> {
        let mut v = vec![];
        for i in 0..self.vertex_indexes.len() {
            let i_next = if i == self.vertex_indexes.len() - 1 { 0 } else { i + 1 };
            let mut vertex_index_pair = [self.vertex_indexes[i], self.vertex_indexes[i_next]];
            vertex_index_pair.sort();
            v.push(vertex_index_pair);
        }
        v
    }

    pub fn to_polygon2_arrays(&self) -> Vec<[f64; 2]> {
        self.points.iter().map(|p| p.to_polygon2_array()).collect()
    }

    pub fn contains_any_point<'a>(polygon_points: &[[f64; 2]], points_iter: impl Iterator<Item = &'a Point>) -> bool {
        for point in points_iter {
            let polygon2_point = &[*point.x, *point.y];
            if polygon2::contains_point(polygon_points, polygon2_point) {
                return true;
            }
        }
        return false;
    }

    pub fn contains_enough_points(polygon_points: &[[f64; 2]], points_to_test: &[[f64; 2]], _rng: &mut ThreadRng) -> bool {
        let points_to_test_count = points_to_test.len();
        /*
        let required_points_count = if points_to_test_count % 2 == 0 {
            points_to_test_count / 2
        } else {
            if rng.gen::<f32>() < 0.5 {
                (points_to_test_count - 1) / 2
            } else {
                (points_to_test_count + 1) / 2
            }
        };
        */
        let required_points_count = points_to_test_count / 2;
        let mut found_points_count = 0;
        for polygon2_point in points_to_test {
            if polygon2::contains_point(polygon_points, polygon2_point) {
                found_points_count += 1;
                if found_points_count >= required_points_count {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn gen_d3(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        self.gen_d3_lines(d3_map, options);
        self.gen_d3_polygon(d3_map, options);
        self.gen_d3_text(d3_map, options);
    }

    fn gen_d3_lines(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        if options.draw_polygon_lines {
            for i in 0..self.points.len() {
                let point_a = &self.points[i];
                let point_b = if i + 1 < self.points.len() {
                    &self.points[i + 1]
                } else {
                    // Wrap around to the first point to complete the last line.
                    &self.points[0]
                };
                let (x1, y1) = point_a.xy_to_d3();
                let (x2, y2) = point_b.xy_to_d3();
                // The key for a line is the concatenated IDs of its endpoints.
                let key = &format!("{}:{}", point_a.id, point_b.id);
                let stroke = options.adjust_color_for_layer(&TERRAIN_EDGE_STROKE, self.layer_type);
                d3_map.lines.push(D3Line::new(key, x1, y1, x2, y2, EDGE_STROKE_WIDTH, &stroke));
            }
        }
    }

    fn gen_d3_polygon(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        if options.draw_polygon_areas {
            // The simplest way for the JS code to read the points is as a single string like:
            //   "200,10 250,190 160,210"
            let points = self.points
                .iter()
                .map(|point| point.xy_to_d3())
                .map(|(x, y)| format!("{},{}", x, y))
                .join(" ");
            // let alpha = 0.25;
            // let fill = format!("rgba({},{},{},{})", rng.gen_range(0, 255), rng.gen_range(0, 255), rng.gen_range(0, 255), alpha);
            let mut fill = match self.polygon_type {
                PolygonType::Ocean => match self.layer_type {
                    LayerType::Terrain => POLYGON_OCEAN_FILL,
                    LayerType::OceanGuide => POLYGON_OCEAN_GUIDE_FILL,
                }
                PolygonType::Land => POLYGON_LAND_FILL,
                PolygonType::Blank => POLYGON_BLANK_FILL,
                _ => POLYGON_UNKNOWN_FILL,
            };
            let stroke_width = 1;
            fill = options.adjust_color_for_layer(&fill, self.layer_type);
            let stroke = fill;
            d3_map.polygons.push(D3Polygon::new(&self.id.to_string(), &points, stroke_width, &stroke, &fill));
        }
    }

    fn gen_d3_text(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        let text = match options.polygon_label_level {
            D3LabelLevel::Altitude => Some(format_f_opt(self.altitude)),
            D3LabelLevel::Sequence => Some(self.sequence.to_string()),
            D3LabelLevel::RelatedSequences => Some(format!("{}\nvertices: {}\nedges: {}",
                                                           self.sequence,
                                                           self.vertex_indexes.iter().join(" ,"),
                                                           self.edge_indexes.iter().join(", "))),
            _ => None,
        };
        if let Some(text) = text {
            let point = self.midpoint();
            d3_map.text.push(D3Text::new(&id().to_string(), &text,
                                         f_to_d3(point.x), f_to_d3(point.y),
                                         DEBUG_LABEL_FONT_FAMILY, DEBUG_LABEL_FONT_SIZE, &DEBUG_LABEL_FILL, DEBUG_LABEL_TEXT_ANCHOR));
        }

    }
}

impl Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let altitude = format_f_opt_labeled(self.altitude, "altitude");
        let area = format_f_opt_labeled(self.area, "area");
        write!(f, "Polygon {{ id: {}, sequence: {}, polygon_type: {:?}{}{}, vertex_indexes: [{}], edge_indexes: [{}], points: [{}] }}",
                self.id,
                self.sequence,
                self.polygon_type,
                altitude,
                area,
                self.vertex_indexes.iter().join(", "),
                self.edge_indexes.iter().join(", "),
                self.points.iter().map(|point| format!("{:?}", point)).join(", "))
    }
}

/*
pub struct Polygon {
    pub id: usize,
    pub sequence: usize,
    pub polygon_type: PolygonType,
    pub points: Vec<Point>,
    pub vertex_indexes: Vec<usize>,
    pub edge_indexes: Vec<usize>,
    pub seed_point: Option<Point>,
    pub area: Option<F>,
}


*/