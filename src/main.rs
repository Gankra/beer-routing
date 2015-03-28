extern crate conrod;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate graphics;
extern crate rand;
extern crate input;

mod graph;
mod dijkstra;

use graph::{Graph, Node};
use dijkstra::Dijkstra;

use conrod::{
    Background,
    Colorable,
    Shapeable,
    Drawable,
    Theme,
    UiContext,
    Point,
    Color,
    Label,
    Positionable,
};
use glutin_window::GlutinWindow;
use opengl_graphics::{Gl, OpenGL, GlGraphics};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event::{
    Event,
    Events,
    Ups,
    MaxFps,
};
use input::{Input, Motion};
use piston::Set;
use piston::window::WindowSettings;
use std::cell::RefCell;
use std::path::Path;
use std::collections::HashSet;

type Ui = UiContext<GlyphCache>;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const NODE_WIDTH: f64 = 45.0;

fn main () {

    let opengl = OpenGL::_3_2;
    let window = GlutinWindow::new(
        opengl,
        WindowSettings {
            title: "Hello Conrod".to_string(),
            size: [WIDTH, HEIGHT],
            fullscreen: false,
            exit_on_esc: true,
            samples: 4,
        }
    );
    let window_ref = RefCell::new(window);
    let mut event_iter: Events<_, _> = Events::new(&window_ref).set(Ups(180)).set(MaxFps(12));
    let mut gl = Gl::new(opengl);
    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let uic = &mut UiContext::new(glyph_cache, theme);

    let rows = 10;
    let graph = graph::gen(rows * 2);
    let start = 45;
    let mut end = 10;

    let mut demo = Demo { first_pass: Dijkstra::new(graph, start), second_pass: None, done: false};
    let mut taken_edges = HashSet::new();
    let mut primary_tree = HashSet::new();

    let canonicalize = |(a, b): (usize, _)| if a < b {
        (a, b)
    } else {
        (b, a)
    };

    for event in &mut event_iter {
        uic.handle_event(&event);
        if let Event::Render(args) = event {
            match progress(&mut demo) {
                None => if !demo.done {
                    primary_tree = taken_edges;
                    taken_edges = HashSet::new();
                } else {
                    break;
                },
                Some(edge) => { taken_edges.insert(canonicalize(edge)); }
            }

            gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
                draw_construction(gl, uic, &demo, &taken_edges);
            });
        }
    }

    let secondary_tree = taken_edges;

    let Demo { first_pass, second_pass, .. } = demo;
    let Dijkstra { graph, predecessors: to_beer, .. } = first_pass;
    let Dijkstra { predecessors: from_beer, .. } = second_pass.unwrap();

    let fake = graph.len() + 1;
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;

    loop {
        let mut cur = end;
        let mut prev = 0;
        let mut path = HashSet::new();

        for event in &mut event_iter {
            uic.handle_event(&event);
            if let Event::Render(args) = event {

                if cur != fake {
                    if let Some(pred) = from_beer[cur] {
                        path.insert(canonicalize((cur, pred)));
                        prev = cur;
                        cur = pred;
                    } else {
                        break; // no path
                    }
                } else {
                    break;
                }

                gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
                    draw_query(gl, uic, &graph, &secondary_tree, &path, start, end);
                });
            }
        }

        cur = prev;
        let mut exited = true;

        for event in &mut event_iter {
            uic.handle_event(&event);
            match event {
                Event::Render(args) => {
                    if let Some(pred) = to_beer[cur] {
                        path.insert(canonicalize((cur, pred)));
                        cur = pred;
                    }

                    gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
                        draw_query(gl, uic,&graph, &primary_tree, &path, start, end);
                    });
                }
                Event::Input(Input::Move(Motion::MouseCursor(x, y))) => {
                    mouse_x = x;
                    mouse_y = y;
                }
                Event::Input(Input::Release(_)) => {
                    let idx = ((mouse_y - 50.0) / 100.0) as usize
                        + (((mouse_x - 50.0) / 100.0) as usize) * rows;

                    if idx < graph.len() {
                        end = idx;
                        exited = false;
                        break;
                    }
                }
                _ => {}
            }
        }

        if exited {
            break;
        }
    }
}

struct Demo {
    first_pass: Dijkstra,
    second_pass: Option<Dijkstra>,
    done: bool,
}

fn progress(demo: &mut Demo) -> Option<(usize, usize)> {
    let Demo { ref mut first_pass, ref mut second_pass, ref mut done } = *demo;

    if second_pass.is_none() {
        let result = first_pass.next();
        if let None = result {
            let num_nodes = first_pass.graph.len();
            let mut augmented = first_pass.graph.clone();

            let mut beer_edges = vec![];
            for (idx, node) in first_pass.graph.iter().enumerate() {
                if node.is_beer {
                    let dist = first_pass.done_distances[idx];
                    beer_edges.push((idx, dist))
                }
            }
            augmented.push(Node{edges: beer_edges, pos: [500.0, 0.0], is_beer: false});

            *second_pass = Some(Dijkstra::new(augmented, num_nodes));
        }

        result
    } else {
        let second_pass = second_pass.as_mut().unwrap();
        let result = second_pass.next();
        if result.is_none() { *done = true; }
        result
    }
}

/// Function for drawing the counter widget.
fn draw_construction(gl: &mut Gl, uic: &mut Ui, demo: &Demo, taken_edges: &HashSet<(usize, usize)>) {
    draw_static(gl, uic);

    let graph = match demo.second_pass {
        None => &demo.first_pass.graph,
        Some(ref second_pass) => &second_pass.graph,
    };

    for (idx, node) in graph.iter().enumerate() {
        for &(edge, _weight) in &node.edges {
            if edge < idx {
                let color = if taken_edges.contains(&(edge, idx)) {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                } else {
                    Color::new(0.8, 0.8, 0.8, 1.0)
                };
                draw_line(gl, node.pos, graph[edge].pos, color);
            }
        }
    }

    for node in graph {
        let color = if node.is_beer {
            Color::new(0.8, 0.0, 0.0, 1.0)
        } else {
            Color::new(0.0, 0.0, 0.8, 1.0)
        };

        draw_circle(gl, node.pos, color);
    }

}

fn draw_query(
    gl: &mut Gl,
    uic: &mut Ui,
    graph: &Graph,
    tree: &HashSet<(usize, usize)>,
    path: &HashSet<(usize, usize)>,
    start: usize,
    end: usize,
) {
    draw_static(gl, uic);

    for (idx, node) in graph.iter().enumerate() {
        for &(edge, _weight) in &node.edges {
            if edge < idx {
                let color = if path.contains(&(edge, idx)) {
                    Color::new(1.0, 0.0, 0.0, 1.0)
                } else if tree.contains(&(edge, idx)) {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                } else {
                    Color::new(0.8, 0.8, 0.8, 1.0)
                };
                draw_line(gl, node.pos, graph[edge].pos, color);
            }
        }
    }

    for (idx, node) in graph.iter().enumerate() {
        let color = if node.is_beer {
            Color::new(0.8, 0.0, 0.0, 1.0)
        } else if idx == start {
            Color::new(0.8, 0.8, 0.0, 1.0)
        } else if idx == end {
            Color::new(0.0, 0.8, 0.8, 1.0)
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        };

        draw_circle(gl, node.pos, color);
    }

}

fn draw_static(gl: &mut Gl, uic: &mut Ui) {
    let font_size = 32;
    let text_x = 1050.0;
    let line_height = 40.0;
    let mut text_y = 40.0;

    Background::new().rgba(1.0, 1.0, 1.0, 1.0).draw(uic, gl);

    let mut line = |text: &'static str| {
        Label::new(text).position(text_x, text_y).size(font_size).draw(uic, gl);
        text_y += line_height;
    };

    line("The Beer Routing Problem:");
    line("You're at home (the source) and you have to go to a");
    line("party (the target). However you need to bring beer");
    line("to the party. Therefore you want to find the shortest");
    line("path from your home to a party that happens to pass");
    line("through some beer store (colored red)");
    line("");

    line("Preprocess Phase 1: Run Dijkstra's Algorithm from the");
    line("source to find the shortest path to each beer store.");
    line("");

    line("Preprocess Phase 2: Insert a fake node connected to");
    line("each beer store with edge-weight equal to the shortest");
    line("path distance from Phase 1. Run Dijkstra again from");
    line("the fake node, producing a forest of neighbourhoods");
    line("around each beer store of target nodes that should");
    line("visit that store.");
    line("");

    line("Query Phase 1: Start at the target node and walk up");
    line("the tree from Preprocess Phase 2 to reach it's beer");
    line("store.");
    line("");

    line("Query Phase 2: Start at the beer store from Phase 1");
    line("and walk up the tree from Preprocess Phase 1 to reach");
    line("the source.");
}

/// Draw a circle controlled by the XYPad.
fn draw_circle(gl: &mut GlGraphics, pos: Point, color: Color) {
    let Color(col) = color;

    graphics::Ellipse::new(col).draw(
        [pos[0], pos[1], NODE_WIDTH, NODE_WIDTH],
        graphics::default_draw_state(),
        graphics::abs_transform(WIDTH as f64, HEIGHT as f64),
        gl
    );
}

fn draw_line(gl: &mut GlGraphics, a: Point, b: Point, color: Color) {
    let Color(col) = color;
    let offset = NODE_WIDTH / 2.0;

    graphics::Line::new(col, 10.0).draw(
        [a[0] + offset, a[1] + offset, b[0] + offset, b[1] + offset],
        graphics::default_draw_state(),
        graphics::abs_transform(WIDTH as f64, HEIGHT as f64),
        gl
    );
}
