use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use rand::Rng;
// use rand::seq::IteratorRandom;
use rand::seq::index;

use super::*;
use rand::prelude::ThreadRng;
use itertools::Itertools;
use std::fmt::{self, Debug};

// This can be used for polygons, vertices, and edges, all of which may be indexed by usize.
#[derive(Eq, PartialEq, Clone, Copy)]
struct ItemDistance {
    dist: F,
    index: usize,
}

impl ItemDistance {
    pub fn new(dist: F, index: usize) -> Self {
        Self { dist, index }
    }
}

impl Ord for ItemDistance {
    fn cmp(&self, other: &ItemDistance) -> Ordering {
        // Flip the usual order and start with _other_ in the next line because we want to get the
        // item with the lowest distance first.
        // (other.dist, other.index).cmp(&(self.dist, self.index))
        (self.dist, self.index).cmp(&(other.dist, other.index))
    }
}

impl PartialOrd for ItemDistance {
    fn partial_cmp(&self, other: &ItemDistance) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Debug for ItemDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ItemDistance {{ dist: {}, index: {} }}", format_f(self.dist), self.index)
    }
}

// This is not a true Iterator since the next() method requires a reference to the PolygonMap.
pub struct VertexDistanceTraverser {
    start_point: Point,
    vertex_queue: BinaryHeap<ItemDistance>,
    noted_vertices: HashSet<usize>,
}

impl VertexDistanceTraverser {
    pub fn new(vertex: &Vertex) -> Self {
        let mut trav = Self {
            start_point: vertex.point.clone(),
            vertex_queue: BinaryHeap::new(),
            noted_vertices: HashSet::new(),
        };
        trav.vertex_queue.push(ItemDistance::new(0.0.into(), vertex.sequence));
        trav.noted_vertices.insert(vertex.sequence);
        trav
    }

    pub fn next(&mut self, layer: &Layer) -> Option<usize> {
        //rintln!("\nnext()\n");
        dbg!(&self);
        // Pull the closest vertex from the queue (which at first will be the starting vertex
        // itself) and add to the queue all of the vertices that this vertex touches and that we
        // haven't seen before.
        if let Some(ItemDistance{ dist: _, index }) = self.vertex_queue.pop() {
            dbg!(index);
            // This is the index of the vertex nearest to the starting point. We want to consider
            // any connected vertices that we haven't already looked at.
            for connected_vertex_index in layer.vertices[index]
                    .connected_vertices(layer)
                    .iter() {
                dbg!(connected_vertex_index);
                if self.noted_vertices.insert(*connected_vertex_index) {
                    // This index was not in the set until now.
                    let dist = self.start_point.distance_to(&layer.vertices[*connected_vertex_index].point);
                    dbg!(&format_f_labeled(dist, "dist"));
                    self.vertex_queue.push(ItemDistance::new(dist, *connected_vertex_index));
                    self.noted_vertices.insert(*connected_vertex_index);
                }
            }
            Some(index)
        } else {
            None
        }
    }
}

impl Debug for VertexDistanceTraverser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vertex_queue = self.vertex_queue
            .clone()
            .into_iter_sorted()
            .map(|item_distance| format!("{:?}", item_distance))
            .join(", ");
        let noted_vertices = self.noted_vertices
            .clone()
            .iter()
            .sorted()
            .join(", ");
        write!(f, "VertexDistanceTraverser {{ start_point: {:?}, vertex_queue: [{}], noted_vertices: [{}] }}",
            self.start_point,
            vertex_queue,
            noted_vertices)
    }
}

pub fn try_vertex_traversal() {
    let mut rng = rand::thread_rng();
    for point_count in 3..=3 {
        let mut map = gen_map(point_count);
        map.terrain.add_voronoi_polygons();
        map.terrain.gen_vertices_and_edges();
        test_vertex_traversal(&map, PCT_100, PCT_100, &mut rng);
    }
}

fn test_vertex_traversal(map: &PolygonMap, test_vertex_pct: Pct, traverse_pct: Pct, rng: &mut ThreadRng) {
    assert!(map.terrain.layer_state.polygons);
    assert!(map.terrain.layer_state.vertices_and_edges);
    let terrain_vertex_count = map.terrain.vertices.len();
    let test_vertex_count = test_vertex_pct * terrain_vertex_count;
    let goal_traversal_size = traverse_pct * terrain_vertex_count;
    let test_vertices = index::sample(rng, terrain_vertex_count, test_vertex_count);
    for test_vertex in test_vertices.iter() {
        // For comparison, create a brute-force version of the traversal list that calculates the
        // distance from this vertex to every other vertex and then sorts the whole list, as
        // opposed to the real iterator that does only the necessary steps, stopping as soon as
        // some condition is met.
        dbg!(&map.terrain.vertices[test_vertex]);

        let start_point = map.terrain.vertices[test_vertex].point.clone();
        let mut dist_list: Vec<ItemDistance> = map.terrain
            .vertices
            .iter()
            .map(|vertex| ItemDistance::new(start_point.distance_to(&vertex.point), vertex.sequence))
            .collect();
        dbg!(&dist_list);

        dist_list.sort();
        dist_list.reverse();
        dbg!(&dist_list);

        let compare_list: Vec<usize> = dist_list.iter()
            .map(|item_distance| item_distance.index)
            .take(goal_traversal_size)
            .collect();
        dbg!(&compare_list);

        let mut trav = VertexDistanceTraverser::new(&map.terrain.vertices[test_vertex]);
        let mut test_list = vec![];
        while let Some(test_vector_index) = trav.next(&map.terrain) {
            test_list.push(test_vector_index);
        }
        dbg!(&test_list);
        assert_eq!(&compare_list, &test_list);
    }
}

/*
fn vertices_by_distance_1(map: &PolygonMap, start_vertex: usize) -> Vec<usize> {
    let start_point = map.terrain.vertices[start_vertex].point.clone();
    let mut dist_list: Vec<ItemDistance> = map.terrain
        .vertices
        .iter()
        .map(|vertex| ItemDistance::new(start_point.distance_to(&vertex.point), vertex.sequence))
        .collect();
    //dbg!(&dist_list);

    dist_list.sort();
    // dist_list.reverse();
    //bg!(&dist_list);

    let compare_list: Vec<usize> = dist_list.iter()
        .map(|item_distance| item_distance.index)
        .collect();
    //bg!(&compare_list);

    compare_list
}
*/

fn all_vertices_by_distance(map: &PolygonMap) {
    for vertex in 0..map.terrain.vertices.len() {
        vertices_by_distance(map, vertex);
    }
}

fn vertices_by_distance(map: &PolygonMap, start_vertex: usize) -> Vec<usize> {
    let start_point = map.terrain.vertices[start_vertex].point.clone();
    map.terrain
        .vertices
        .iter()
        .map(|vertex| ItemDistance::new(start_point.distance_to(&vertex.point), vertex.sequence))
        .sorted()
        .map(|item_distance| item_distance.index)
        .collect()
}

pub fn time_vertex_traversal() {
    for point_count in crate::sort::test_data::vec_powers(10, 10, 2) {
        let mut map = gen_map(point_count);
        map.terrain.add_voronoi_polygons();
        map.terrain.gen_vertices_and_edges();

        //let start_vertex = point_count / 2;
        //let v1 = vectors_by_distance_1(&map, start_vertex);
        //let v = vectors_by_distance(&map, start_vertex);
        //dbg!(&v1, &v);
        //assert_eq!(v1, v);

        util::format::print_elapsed(true, &point_count.to_string(), "",
                                    || { all_vertices_by_distance(&map); } );

    }
}

// This can be used for polygons, vertices, and edges, all of which may be indexed by usize.
#[derive(Eq, PartialEq, Clone, Copy)]
pub struct ItemStepDistance {
    pub dist: F,
    pub step_count: usize,
    pub index: usize,
    pub complete: bool,
}

impl ItemStepDistance {
    pub fn new(dist: F, step_count: usize, index: usize, complete: bool) -> Self {
        Self { dist, step_count, index, complete }
    }
}

impl Ord for ItemStepDistance {
    fn cmp(&self, other: &ItemStepDistance) -> Ordering {
        // Flip the usual order and start with _other_ in the next line because we want to get the
        // item with the lowest distance first.
        (other.dist, other.index).cmp(&(self.dist, self.index))
    }
}

impl PartialOrd for ItemStepDistance {
    fn partial_cmp(&self, other: &ItemStepDistance) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Debug for ItemStepDistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ItemStepDistance {{ dist: {}, step_count: {}, index: {}, complete: {} }}",
            format_f(self.dist),
            self.step_count,
            self.index,
            self.complete)
    }
}

// This is not a true Iterator since the next() method requires a reference to the PolygonMap.
pub struct VertexStepDistanceTraverser {
    dist_mult: F,
    dist_add: F,
    vertex_queue: BinaryHeap<ItemStepDistance>,
    noted_vertices: HashSet<usize>,
}

impl VertexStepDistanceTraverser {
    pub fn new(dist_mult: F, dist_add: F, vertex_index: usize) -> Self {
        let mut trav = Self {
            dist_mult,
            dist_add,
            vertex_queue: BinaryHeap::new(),
            noted_vertices: HashSet::new(),
        };
        trav.vertex_queue.push(ItemStepDistance::new(0.0.into(),0, vertex_index, false));
        trav.noted_vertices.insert(vertex_index);
        trav
    }

    pub fn next(&mut self, layer: &Layer, short_circuit: bool) -> Option<ItemStepDistance> {
        //rintln!("\nnext()\n");
        //bg!(&self);
        // Pull the closest vertex from the queue (which at first will be the starting vertex
        // itself) and add to the queue all of the vertices that this vertex touches and that we
        // haven't seen before.
        if let Some(item_step_distance) = self.vertex_queue.pop() {
            if short_circuit && item_step_distance.complete {
                return Some(item_step_distance);
            }
            //bg!(&item_step_distance);
            // This is the vertex nearest to the starting point. We want to consider any connected
            // vertices that we haven't already looked at.
            let this_vertex: &Vertex = &layer.vertices[item_step_distance.index];
            for edge_index in this_vertex.edges.iter() {
                let connected_edge: &Edge = &layer.edges[*edge_index];
                //bg!(&connected_edge);
                let connected_vertex_index: usize = connected_edge.other_vertex(item_step_distance.index);
                if self.noted_vertices.insert(connected_vertex_index) {
                    // This index was not in the set until now.
                    // let connected_vertex: &Vertex = &layer.vertices[connected_vertex_index];
                    //bg!(&connected_vertex);
                    let connected_edge_length = connected_edge.length;
                    let dist = item_step_distance.dist + (connected_edge_length * self.dist_mult) + self.dist_add;
                    let step_count = item_step_distance.step_count + 1;
                    let connected_vertex: &Vertex = &layer.vertices[connected_vertex_index];
                    let new_item_step_distance = if short_circuit && connected_vertex.steps_to_water.is_some() {
                        ItemStepDistance::new(item_step_distance.dist + connected_vertex.distance_to_water.unwrap(), item_step_distance.step_count + connected_vertex.steps_to_water.unwrap(), connected_vertex_index, true)
                    } else {
                        ItemStepDistance::new(dist, step_count, connected_vertex_index, false)
                    };
                    //bg!(&new_item_step_distance);
                    self.vertex_queue.push(new_item_step_distance);
                }
            }
            Some(item_step_distance)
        } else {
            None
        }
    }
}

impl Debug for VertexStepDistanceTraverser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vertex_queue = self.vertex_queue
            .clone()
            .into_iter_sorted()
            .map(|item_step_distance| format!("{:?}", item_step_distance))
            .join(", ");
        let noted_vertices = self.noted_vertices
            .clone()
            .iter()
            .sorted()
            .join(", ");
        write!(f, "VertexStepDistanceTraverser {{ dist_mult: {}, dist_add: {}, vertex_queue: [{}], noted_vertices: [{}] }}",
               format_f(self.dist_mult),
               format_f(self.dist_add),
               vertex_queue,
               noted_vertices)
    }
}

pub fn try_vertex_step_traversal() {
    let mut rng = rand::thread_rng();
    for point_count in 3..=3 {
        let mut map = gen_map(point_count);
        map.terrain.add_voronoi_polygons();
        map.terrain.gen_vertices_and_edges();
        for dist_add in [f(0.03), f(0.06), f(0.12), f(0.24)].iter() {
            test_vertex_step_traversal(&map, f(1.0), *dist_add, PCT_100, PCT_100, &mut rng);
        }
        // Set dist_mult to 0 and dist_add to 1 so that the distance is simply the count of steps.
        test_vertex_step_traversal(&map, f(0.0), f(1.0), PCT_100, PCT_100, &mut rng);
    }
}

fn test_vertex_step_traversal(map: &PolygonMap, dist_mult: F, dist_add: F, test_vertex_pct: Pct, traverse_pct: Pct, rng: &mut ThreadRng) {
    assert!(map.terrain.layer_state.polygons);
    assert!(map.terrain.layer_state.vertices_and_edges);
    let terrain_vertex_count = map.terrain.vertices.len();
    let test_vertex_count = test_vertex_pct * terrain_vertex_count;
    let _goal_traversal_size = traverse_pct * terrain_vertex_count;
    let test_vertices = index::sample(rng, terrain_vertex_count, test_vertex_count);
    for test_vertex_index in test_vertices.iter() {
        let mut trav = VertexStepDistanceTraverser::new(dist_mult, dist_add, test_vertex_index);
        let mut test_list = vec![];
        while let Some(item_step_distance) = trav.next(&map.terrain, false) {
            test_list.push(item_step_distance);
        }
        dbg!(&test_list);
    }
}

pub fn try_distance_to_water() {
    for point_count in crate::sort::test_data::vec_powers(10, 10, 2) {
        let mut map = try_ocean_multi_level(point_count);
        util::format::print_elapsed(true, &point_count.to_string(), "", || {
            map.terrain.calc_distance_to_water(f(1.0), f(0.05), false);
        });
        util::format::print_elapsed(true, &point_count.to_string(), "", || {
            map.terrain.calc_distance_to_water(f(1.0), f(0.05), true);
        });
    }
}

