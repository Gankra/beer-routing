use conrod::Point;
use rand::{self, Rng};

pub type Graph = Vec<Node>;

#[derive(Clone)]
pub struct Node {
    pub is_beer: bool,
    pub pos: Point,
    pub edges: Vec<(usize, f64)>,
}

pub fn gen(size: usize) -> Graph {
    let mut nodes: Vec<Node> = Vec::with_capacity(size);

    let row_width = size/2;

    let mut rng = rand::thread_rng();

    for i in 1..row_width+1 {
        for j in 1..row_width+1 {
            let x = (i * 100 - 50) as f64;
            let y = (j * 100 - 50) as f64;
            let is_beer = rng.next_f64() < 0.05;
            let mut edges = vec![];

            {
                let mut add_edge = |xc, yc| {
                    if rng.next_f64() < 0.25 { return; }
                    if xc < 1 || xc > row_width || yc < 1 || yc > row_width { return; }
                    let idx = (xc - 1) * row_width + (yc - 1);
                    let num_nodes = nodes.len();
                    if idx >= num_nodes { return; }
                    let other = &mut nodes[idx];
                    let dist = ((x - other.pos[0]).powi(2) + (y - other.pos[1]).powi(2)).sqrt();
                    edges.push((idx, dist));
                    other.edges.push((num_nodes, dist));
                };

                add_edge(i-1, j);
                add_edge(i+1, j);
                add_edge(i, j-1);
                add_edge(i, j+1);
            }


            nodes.push(Node {
                is_beer: is_beer,
                pos: [x, y],
                edges: edges,
            })
        }
    }

    nodes
}
