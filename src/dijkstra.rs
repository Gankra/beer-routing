use graph::Graph;
use std::f64;

pub struct Dijkstra {
    pub graph: Graph,
    pub todo_distances: Vec<Option<f64>>,
    pub predecessors: Vec<Option<usize>>,
    pub done_distances: Vec<f64>,
    pub current: usize,
}

impl Dijkstra {
    pub fn new(input: Graph, start: usize) -> Dijkstra {
        let nodes = input.len();

        let mut result = Dijkstra {
            graph: input,
            todo_distances: vec![Some(f64::INFINITY); nodes],
            done_distances: vec![0.0; nodes],
            predecessors: vec![None; nodes],
            current: start,
        };

        result.todo_distances[start] = None;
        result
    }
}

impl Iterator for Dijkstra {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<(usize, usize)> {
        let Dijkstra {
            ref graph,
            ref mut todo_distances,
            ref mut predecessors,
            ref mut done_distances,
            ref mut current,
        } = *self;

        let cur_node = *current;
        let cur_dist = done_distances[cur_node];
        for &(node, weight) in &graph[cur_node].edges {
            todo_distances[node].map(|best_dist| {
                let new_dist = cur_dist + weight;
                if new_dist < best_dist {
                    predecessors[node] = Some(cur_node);
                    todo_distances[node] = Some(new_dist);
                }
            });
        }

        let mut min_dist = f64::INFINITY;
        let mut min_node = None;
        for (node, &dist) in todo_distances.iter().enumerate() {
            dist.map(|dist| {
                if dist < min_dist {
                    min_dist = dist;
                    min_node = Some(node);
                }
            });
        }

        min_node.map(|next_node| {
            let pred = predecessors[next_node].unwrap();
            *current = next_node;
            done_distances[next_node] = min_dist;
            todo_distances[next_node] = None;
            (pred, next_node)
        })
    }
}

/*
pub fn beer_route(graph: Graph, start: usize) -> (Vec<Option<usize>>, Vec<Option<usize>>) {
    let mut first_pass = Dijkstra::new(graph, start);
    for _ in &mut first_pass {}

    let num_nodes = first_pass.graph.len();
    let mut augmented = first_pass.graph.clone();

    let mut beer_edges = vec![];
    for (idx, node) in graph.iter().enumerate() {
        if node.is_beer {
            let dist = first_pass.done_distances[idx];
            beer_edges.push((idx, dist))
        }
    }
    augmented.push(Node{edges: beer_edges, pos: [0.0, 0.0], is_beer: false});

    let mut second_pass = Dijkstra::new(augmented, num_nodes);
    for _ in &mut second_pass {}

    (first_pass.predecessors, second_pass.predecessors)
}

pub fn find_path(graph: &Graph, start: usize, end: usize) -> Vec<usize> {
    let (to_beer, from_beer) = beer_route(graph.clone(), start);

    let mut path = vec![];
    let fake = from_beer.len();
    let mut cur = end;
    while cur != fake {
        path.push(cur);
        if let Some(pred) = from_beer[cur] {
            cur = pred;
        } else {
            return vec![]; // no path
        }
    }
    cur = *path.last().unwrap();
    while let Some(pred) = to_beer[cur] {
        path.push(pred);
    }

    path
}
*/
