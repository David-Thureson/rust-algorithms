use std::collections::BTreeSet;
use std::fmt::{self, Debug};
use itertools::Itertools;

use super::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy)]
pub enum VertexType {
    Unknown,
    Land,
    Peak,
    Coast,
    Ocean,
}

#[derive(Clone)]
pub struct Vertex {
    pub id: usize,
    pub sequence: usize,
    pub vertex_type: VertexType,
    pub point: Point,
    pub polygons: Vec<Rc<RefCell<Polygon>>>,
    pub edges: Vec<Rc<RefCell<Edge>>>,
    pub distance_to_water: Option<F>,
    pub altitude: Option<F>,
}

impl Vertex {
    pub fn new(sequence: usize, point: &Point) -> Self {
        Self {
            id: id(),
            sequence,
            vertex_type: VertexType::Unknown,
            point: point.clone(),
            polygons: vec![],
            edges: vec![],
            distance_to_water: None,
            altitude: None,
        }
    }

    pub fn rotate(&mut self, axis_point: &Point) {
        self.point.rotate(axis_point);
    }

    pub fn gen_d3(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        self.gen_d3_text(d3_map, options);
    }

    fn gen_d3_text(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {
        let text = match options.vertex_label_level {
            D3LabelLevel::Altitude => Some(format_f_opt(self.altitude)),
            D3LabelLevel::Sequence => Some(self.sequence.to_string()),
            D3LabelLevel::RelatedSequences => Some(format!("{}: polygons: {}, edges: {}",
                                                      self.sequence,
                                                      self.polygon_indexes.iter().join(" ,"),
                                                      self.edge_indexes.iter().join(", "))),
            _ => None,
        };
        if let Some(text) = text {
            d3_map.text.push(D3Text::new(&id().to_string(), &text,
                     f_to_d3(self.point.x), f_to_d3(self.point.y),
                     DEBUG_LABEL_FONT_FAMILY, DEBUG_LABEL_FONT_SIZE, &DEBUG_LABEL_FILL, DEBUG_LABEL_TEXT_ANCHOR));
        }

    }

}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.sequence.cmp(*other.sequence)
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let altitude = if let Some(altitude) = self.altitude { format!(", altitude: {}", format_f(altitude)) } else { "".to_string() };
        let altitude = format_f_labeled(self.altitude, "altitude");
        write!(f, "Vertex {{ id: {}, sequence: {}, vertex_type: {:?}, point: {:?}, polygon_indexes: [{}], edge_indexes: [{}]{} }}",
                self.id,
                self.sequence,
                self.vertex_type,
                self.point,
                self.polygon_indexes.iter().join(", "),
                self.edge_indexes.iter().join(", "),
                altitude)
    }
}

