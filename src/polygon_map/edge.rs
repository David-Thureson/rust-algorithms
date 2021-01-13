use std::fmt::{self, Debug};
use std::collections::BTreeSet;
use crate::itertools::Itertools;

use super::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    Unknown,
    Land,
    Ridge,
    Coast,
    Ocean,
}

#[derive(Clone)]
pub struct Edge {
    pub id: usize,
    pub sequence: usize,
    pub edge_type: EdgeType,
    pub layer_type: LayerType,
    pub points: [Point; 2],
    pub length: F,
    pub altitude: Option<F>,
    pub polygons: Vec<Rc<RefCell<Polygon>>>,
    pub vertices: Vec<Rc<RefCell<Vertex>>>,
}

impl Edge {
    pub fn new(sequence: usize, layer_type: LayerType, points: [&Point; 2]) -> Self {
        let mut points_cloned = [points[0].clone(), points[1].clone()];
        points_cloned.sort();
        Self {
            id: id(),
            sequence,
            edge_type: EdgeType::Unknown,
            layer_type,
            points: points_cloned,
            length: points[0].distance_to(&points[1]),
            altitude: None,
            polygons: vec![],
            vertices: vec![],
        }
    }

    pub fn midpoint(&self) -> Point {
        Point::mean(self.points.iter())
    }

    pub fn rotate(&mut self, axis_point: &Point) {
        for point in self.points.iter_mut() {
            point.rotate(axis_point);
        }
    }

    pub fn other_vertex(&self, rc_vertex: Rc<RefCell<Vertex>>) -> Rc<RefCell<Vertex>> {
        assert_eq!(2, self.vertices.len());
        let vertex: &RefCell<Vertex> = Rc::borrow(&rc_vertex);
        let vertex_0: &RefCell<Vertex> = Rc::borrow(&self.vertices[0]);
        // if Rc::borrow(rc_vertex).eq(Rc::borrow(self.vertices[0]))  {
        if vertex.eq(&vertex_0) {
            Rc::clone(&self.vertices[1])
        } else {
            Rc::clone(&self.vertices[0])
        }
        /*
        for rc_vertex_other in self.vertices.iter() {
            if Rc::borrow(rc_vertex_other) != Rc::borrow(&rc_vertex) {
                return Rc::clone(rc_vertex_other);
            }
        }
        unreachable!()
        */
    }

    pub fn gen_d3(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        self.gen_d3_line(d3_map, options);
        self.gen_d3_text(d3_map, options);
    }

    fn gen_d3_line(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        if options.draw_edges {
            let mut stroke = match self.layer_type {
                LayerType::Terrain => TERRAIN_EDGE_STROKE,
                LayerType::OceanGuide => OCEAN_GUIDE_EDGE_STROKE,
            };
            stroke = options.adjust_edge_opacity_for_labels(&stroke);
            stroke = options.adjust_color_for_layer(&stroke, self.layer_type);
            let (x1, y1) = self.points[0].xy_to_d3();
            let (x2, y2) = self.points[1].xy_to_d3();
            d3_map.lines.push(D3Line::new(&self.id.to_string(), x1, y1, x2, y2, EDGE_STROKE_WIDTH, &stroke));
        }
    }

    fn gen_d3_text(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        let text = match options.edge_label_level {
            D3LabelLevel::Altitude => Some(format_f_opt(self.altitude)),
            D3LabelLevel::Sequence => Some(self.sequence.to_string()),
            D3LabelLevel::RelatedSequences => Some(format!("{}\npolygons: {}\nvertices: {}",
                                                           self.sequence,
                                                           self.polygon_indexes.iter().join(" ,"),
                                                           self.vertex_indexes.iter().join(", "))),
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

impl Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let altitude = format_f_labeled(self.altitude, "altitude");
        write!(f, "Vertex {{ id: {}, sequence: {}, edge_type: {:?}{}, polygon_indexes: [{}], vertex_indexes: [{}] }}",
                self.id,
                self.sequence,
                self.edge_type,
                altitude,
                self.polygon_indexes.iter().join(", "),
                self.vertex_indexes.iter().join(", "))
    }
}



