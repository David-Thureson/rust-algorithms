use std::fmt::{self, Debug};
use itertools::Itertools;

use super::*;

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
    pub polygons: Vec<usize>,
    pub edges: Vec<usize>,
    pub steps_to_water: Option<usize>,
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
            steps_to_water: None,
            distance_to_water: None,
            altitude: None,
        }
    }

    pub fn rotate(&mut self, axis_point: &Point) {
        self.point.rotate(axis_point);
    }

    pub fn connected_vertices(&self, layer: &Layer) -> Vec<usize> {
        self.edges
            .iter()
            .map(|edge_index| layer.edges[*edge_index].other_vertex(self.sequence))
            .collect()
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
                                                      self.polygons.iter().join(" ,"),
                                                      self.edges.iter().join(", "))),
            _ => None,
        };
        if let Some(text) = text {
            d3_map.text.push(D3Text::new(&id().to_string(), &text,
                     f_to_d3(self.point.x), f_to_d3(self.point.y),
                     DEBUG_LABEL_FONT_FAMILY, DEBUG_LABEL_FONT_SIZE, &DEBUG_LABEL_FILL, DEBUG_LABEL_TEXT_ANCHOR));
        }

    }

}

impl Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let altitude = if let Some(altitude) = self.altitude { format!(", altitude: {}", format_f(altitude)) } else { "".to_string() };
        let altitude= format_f_opt_labeled(self.altitude, "altitude");
        let steps_to_water= format_count_opt_labeled(self.steps_to_water, "steps_to_water");
        let distance_to_water= format_f_opt_labeled(self.distance_to_water, "distance_to_water");
        write!(f, "Vertex {{ id: {}, sequence: {}, vertex_type: {:?}, point: {:?}{}{}{}, polygon_indexes: [{}], edge_indexes: [{}] }}",
            self.id,
            self.sequence,
            self.vertex_type,
            self.point,
            altitude,
            steps_to_water,
            distance_to_water,
            self.polygons.iter().join(", "),
            self.edges.iter().join(", "))
    }
}

