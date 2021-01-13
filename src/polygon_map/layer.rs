use std::collections::{HashMap, BTreeMap};
use itertools::Itertools;

use super::*;

use rand::Rng;
use std::convert::Into;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy)]
pub enum LayerType {
    Terrain,
    OceanGuide,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: usize,
    pub layer_type: LayerType,
    pub layer_state: LayerState,
    pub points: Vec<Point>,
    pub polygons: Vec<Rc<RefCell<Polygon>>>,
    pub vertices: Vec<Rc<RefCell<Vertex>>>,
    pub edges: Vec<Rc<RefCell<Edge>>>,
}

impl Layer {
    pub fn new(layer_type: LayerType) -> Self {
        Self {
            id: id(),
            layer_type,
            layer_state: LayerState::new(),
            points: vec![],
            polygons: vec![],
            vertices: vec![],
            edges: vec![],
        }
    }

    pub fn take_to_pre_water_state(&mut self, goal_state: &LayerState, force_new: bool) -> bool {
        let mut changed = false;

        if force_new || goal_state.point_count != self.layer_state.point_count {
            self.gen_points(goal_state.point_count);
            changed = true;
        }

        if goal_state.polygons && !self.layer_state.polygons {
            self.add_voronoi_polygons();
            changed = true;
        }

        if goal_state.polygons {
            for _ in 0..(goal_state.relax_level - self.layer_state.relax_level) {
                self.relax_polygons();
                changed = true;
            }

            if goal_state.vertices_and_edges && !self.layer_state.vertices_and_edges {
                self.gen_vertices_and_edges();
                changed = true;
            }
        }

        assert_eq!(goal_state.point_count, self.layer_state.point_count);
        assert_eq!(goal_state.polygons, self.layer_state.polygons);
        // It's possible that the caller asked for a lower relax state in which case we did
        // nothing.
        assert!(goal_state.relax_level <= self.layer_state.relax_level);
        assert_eq!(goal_state.vertices_and_edges, self.layer_state.vertices_and_edges);

        changed
    }

    pub fn gen_points(&mut self, point_count: usize) {
        self.points.clear();
        self.polygons.clear();
        self.vertices.clear();
        self.edges.clear();
        self.layer_state = LayerState::new();
        let mut rng = rand::thread_rng();
        for _ in 0..point_count {
            self.points.push(Point::new(
                rng.gen::<f64>().into(),
                rng.gen::<f64>().into(),
            ));
        }
        self.layer_state.point_count = point_count;
    }

    pub fn add_voronoi_polygons(&mut self) {
        self.polygons.clear();
        // let vor_pts = self.vor_points.iter().map(|p| voronoi::Point::new(p.x, p.y)).collect();
        let vor_points = self.points.iter().map(|point| point.to_voronoi_point()).collect();
        let vor_diagram = voronoi::voronoi(vor_points, 1.0);
        let vor_polys = voronoi::make_polygons(&vor_diagram);
        // assert_eq!(vor_polys.len(), self.vor_points.len());
        //bg!(&self.vor_points);
        for (index, vor_poly) in vor_polys.iter().enumerate() {

            let seed_point = None;

            let mut polygon = Polygon::new(index, PolygonType::Unknown, self.layer_type, seed_point);
            polygon.points = vor_poly.iter().map(|p| Point::from_voronoi_point(p)).collect();
            self.polygons.push(polygon);
        }
        // self.set_water(0.40.into());
        self.layer_state.polygons = true;
    }

    pub fn relax_polygons(&mut self) {
        self.polygons.clear();
        let vor_points = self.points.iter().map(|point| point.to_voronoi_point()).collect();
        let new_vor_points = voronoi::lloyd_relaxation(vor_points, 1.0);
        for (index, new_point) in new_vor_points.iter().enumerate() {
            if index < self.points.len() {
                self.points[index].update_from_voronoi_point(new_point);
            }
        }
        self.add_voronoi_polygons();
        self.layer_state.relax_level += 1;
    }

    pub fn calc_polygon_areas(&mut self) {
        for polygon in self.polygons.iter_mut() {
            polygon.calc_area();
        }
    }

    pub fn set_water_from_pct(&mut self, pct: F) {
        // This function fills some percentage of area using the largest polygons within the layer,
        // and is used for setting up the ocean guide layer. This is not the function that
        // determines water polygons on the _terrain_ layer by looking at the ocean layer.
        // assert!(self.layer_type == LayerType::OceanGuide);
        self.calc_polygon_areas();
        // Since the overall map is 1.0 x 1.0, the target area is the same as the percent to cover
        // with water.
        let target_area = pct;
        let mut area_so_far: F = 0.0.into();
        for polygon in self.polygons
            .iter_mut()
            // .sorted_by(|a, b| float_value_nans_panic_f64(0.0 - a.area.unwrap(), 0.0 - b.area.unwrap())) {
            .sorted_by_key(|point| point.area.unwrap()) {
            if area_so_far < target_area {
                polygon.polygon_type = PolygonType::Ocean;
                area_so_far += polygon.area.unwrap();
            } else {
                polygon.polygon_type = PolygonType::Land;
            }
        }
        self.layer_state.water_pct = pct;
    }

    pub fn clear_water(&mut self) {
        // On the terrain layer, get rid of any water information since we've changed something
        // about the ocean guide layer.
        // assert!(self.layer_type == LayerType::Terrain);
        for polygon in self.polygons.iter_mut() {
            polygon.polygon_type = PolygonType::Blank;
        }
        self.layer_state.water = false;
    }

    pub fn gen_vertices_and_edges(&mut self) {

        self.vertices.clear();
        self.edges.clear();

        // Make sure the sequence numbers for polygons match their place in the vector.
        for (index, polygon) in self.polygons.iter().enumerate() {
            assert_eq!(index, polygon.sequence);
        }

        let mut vertex_map: HashMap<String, usize> = HashMap::new();
        let mut edge_map: HashMap<(usize, usize), usize> = HashMap::new();
        let mut next_vertex_sequence = 0;
        let mut next_edge_sequence = 0;

        for polygon in self.polygons.iter_mut() {
            polygon.vertex_indexes.clear();
            polygon.edge_indexes.clear();

            for point in polygon.points.iter_mut() {
                // Find the vertex or add one.
                let vertex_key = point.key();
                let vertex_index = if vertex_map.contains_key(&vertex_key) {
                    *vertex_map.get(&vertex_key).unwrap()
                } else {
                    vertex_map.insert(vertex_key, self.vertices.len());
                    self.vertices.push(Vertex::new(next_vertex_sequence, &point));
                    next_vertex_sequence += 1;
                    next_vertex_sequence - 1
                };
                let was_added= self.vertices[vertex_index].polygon_indexes.insert(polygon.sequence);
                assert!(was_added);
                polygon.vertex_indexes.push(vertex_index);
            }
            assert_eq!(polygon.vertex_indexes.len(), polygon.points.len());

            for vertex_index_pair in polygon.vertex_index_ordered_pairs() {
                let edge_key = (vertex_index_pair[0], vertex_index_pair[1]);
                let edge_index = if edge_map.contains_key(&edge_key) {
                    *edge_map.get(&edge_key).unwrap()
                } else {
                    edge_map.insert(edge_key, self.edges.len());
                    let mut new_edge = Edge::new(
                        next_edge_sequence,
                        self.layer_type,
                        [
                        &self.vertices[vertex_index_pair[0]].point,
                        &self.vertices[vertex_index_pair[1]].point
                        ]);

                    // Tie the edge and its vertexes together.
                    for vertex_index in vertex_index_pair.iter() {
                        // if self.vertices[*vertex_index].edge_indexes.contains(&edge_index) {
                        //     dbg!(&polygon, &vertex_index_pair, &edge_key, &edge_index, &vertex_index, &self.vertices[*vertex_index], &self.edges[edge_index]);
                        // }
                        let was_added= self.vertices[*vertex_index].edge_indexes.insert(next_edge_sequence);
                        assert!(was_added);
                        let was_added= new_edge.vertex_indexes.insert(*vertex_index);
                        assert!(was_added);
                    }

                    self.edges.push(new_edge);
                    next_edge_sequence += 1;
                    next_edge_sequence - 1
                };
                assert!(self.edges[edge_index].polygon_indexes.len() < 2);
                let was_added = self.edges[edge_index].polygon_indexes.insert(polygon.sequence);
                assert!(was_added);
                polygon.edge_indexes.push(edge_index);
            }
            assert_eq!(polygon.edge_indexes.len(), polygon.points.len());

        }
        // Make sure the sequence numbers for vertices match their place in the vector.
        for (index, vertex) in self.vertices.iter().enumerate() {
            assert_eq!(index, vertex.sequence);
        }
        // Make sure the sequence numbers for edges match their place in the vector.
        for (index, edge) in self.edges.iter().enumerate() {
            assert_eq!(index, edge.sequence);
        }

        for polygon in self.polygons.iter_mut() {
            polygon.polygon_type = PolygonType::Blank;
        }

        self.layer_state.vertices_and_edges = true;
    }

    pub fn calc_distance_to_water(&mut self) {
        for polygon in self.polygons.iter_mut() {
            polygon.altitude = None;
        }
        for vertex in self.vertices.iter_mut() {
            vertex.vertex_type = VertexType::Unknown;
            vertex.distance_to_water = None;
            vertex.altitude = None;
        }
        for edge in self.edges.iter_mut() {
            edge.edge_type = EdgeType::Unknown;
            edge.altitude = None;
        }
        for vertex_index in 0..self.vertices.len() {
            let ocean_count = self.vertex_ocean_count(vertex_index);
            if ocean_count > 0 {
                let mut vertex = &mut self.vertices[vertex_index];
                vertex.vertex_type = if vertex.polygon_indexes.len() == ocean_count {
                    VertexType::Ocean
                } else {
                    VertexType::Coast
                };
                vertex.altitude = Some(0.0.into());
            } else {
                let starting_point = self.vertices[vertex_index].point.clone();
                // Use the BTreeMap as a priority queue. At each step add the vertices attached
                // to the current vertex, then process the first vertex in the queue which will be
                // the one nearest the original vertex.
                let mut map: BTreeMap<F, usize> = BTreeMap::new();
                let mut one_vertex_index = vertex_index;
                let mut found = false;
                while !found {
                    for connected_vertex_index in self.vertices[one_vertex_index].edge_indexes
                            .iter()
                            .map(|edge_index| self.edges[*edge_index].other_vertex_index(one_vertex_index)) {
                        let distance = starting_point.distance_to(&self.vertices[connected_vertex_index].point);
                        map.insert(distance, self.vertices[connected_vertex_index].sequence);
                    }
                    let (new_vertex_distance, new_vertex_index) = {
                        let (key, val) = map.first_key_value().unwrap();
                        (*key, *val)
                    };
                    map.remove(&new_vertex_distance);
                    one_vertex_index = new_vertex_index;
                    if self.vertex_ocean_count(one_vertex_index) > 0 {
                        self.vertices[vertex_index].vertex_type = VertexType::Land;
                        self.vertices[vertex_index].distance_to_water = Some(new_vertex_distance);
                        found = true;
                    }
                }
            }
        }
    }

    fn vertex_ocean_count(&self, vertex_index: usize) -> usize {
        self.vertices[vertex_index]
            .polygon_indexes
            .iter()
            .map(|polygon_index| self.polygons[*polygon_index].polygon_type.clone())
            .filter(|polygon_type| *polygon_type == PolygonType::Ocean)
            .count()
    }

    pub fn rotate(&mut self, axis_point: &Point) {
        for point in self.points.iter_mut() {
            point.rotate(axis_point);
        }
        for polygon in self.polygons.iter_mut() {
            polygon.rotate(axis_point);
        }
        for vertex in self.vertices.iter_mut() {
            vertex.rotate(axis_point);
        }
        for edge in self.edges.iter_mut() {
            edge.rotate(axis_point);
        }
    }

    pub fn gen_d3(&self, d3_map: &mut D3Map, options: &mut D3MapOptions) {

        for point in self.points.iter() {
            point.gen_d3(d3_map, options);
        }

        for polygon in self.polygons.iter() {
            polygon.gen_d3(d3_map, options);
        }

        for vertex in self.vertices.iter() {
            vertex.gen_d3(d3_map, options);
        }

        for edge in self.edges.iter() {
            edge.gen_d3(d3_map, options);
        }
    }

    pub fn all_points(&self) -> Vec<&Point> {
        self.points.iter().chain(self.polygons.iter().map(|polygon| polygon.points.iter()).flatten()).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LayerState {
    pub point_count: usize,
    pub polygons: bool,
    pub relax_level: u8,
    pub vertices_and_edges: bool,
    pub water_pct: F,
    pub water: bool,
    pub altitudes: bool,
    pub coloring: bool,
}

impl LayerState {
    pub(crate) fn new() -> Self {
        Self {
            point_count: 0,
            polygons: false,
            relax_level: 0,
            vertices_and_edges: false,
            water_pct: 0.0.into(),
            water: false,
            altitudes: false,
            coloring: false,
        }
    }
}

