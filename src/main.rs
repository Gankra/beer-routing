extern crate conrod;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate graphics;

mod graph;
mod dijkstra;

use graph::Node;
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
use piston::Set;
use piston::window::WindowSettings;
use std::cell::RefCell;
use std::path::Path;
use std::collections::HashSet;

type Ui = UiContext<GlyphCache>;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const NODE_WIDTH: f64 = 60.0;

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
    let event_iter: () = Events::new(&window_ref).set(Ups(180)).set(MaxFps(60));
    let mut gl = Gl::new(opengl);
    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let uic = &mut UiContext::new(glyph_cache, theme);

    let graph = graph::gen(20);

    let mut elapsed_frames = 0;
    let mut demo = Demo { first_pass: Dijkstra::new(graph, 3), second_pass: None};
    let mut taken_edges = HashSet::new();

    for event in event_iter {
        uic.handle_event(&event);
        if let Event::Render(args) = event {
            elapsed_frames += 1;
            if elapsed_frames > 5 {
                elapsed_frames = 0;
                match progress(&mut demo) {
                    None => { taken_edges = HashSet::new(); }
                    Some((a, b)) => if a < b {
                        taken_edges.insert((a, b));
                    } else {
                        taken_edges.insert((b, a));
                    }
                }
            }
            gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
                draw(gl, uic, &demo, &taken_edges);
            });
        }
    }
}

struct Demo {
    first_pass: Dijkstra,
    second_pass: Option<Dijkstra>,
}

fn progress(demo: &mut Demo) -> Option<(usize, usize)> {
    let Demo { ref mut first_pass, ref mut second_pass } = *demo;

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
            augmented.push(Node{edges: beer_edges, pos: [0.0, 0.0], is_beer: false});

            *second_pass = Some(Dijkstra::new(augmented, num_nodes));
        }

        result
    } else {
        let second_pass = second_pass.as_mut().unwrap();
        second_pass.next()
    }
}

/// Function for drawing the counter widget.
fn draw(gl: &mut Gl, uic: &mut Ui, demo: &Demo, taken_edges: &HashSet<(usize, usize)>) {

    // Draw the background.
    Background::new().rgba(1.0, 1.0, 1.0, 1.0).draw(uic, gl);

    // Draw the value.
    // Label::new("1").position(10.0, 10.0).draw(uic, gl);

    let graph = match demo.second_pass {
        None => &demo.first_pass.graph,
        Some(ref second_pass) => &second_pass.graph,
    };

    for (idx, node) in graph.iter().enumerate() {
        for &(edge, _weight) in &node.edges {
            if edge < idx {
                let color = if taken_edges.contains(&(edge, idx)) {
                    Color::new(0.0, 0.8, 0.0, 1.0)
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
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

    graphics::Line::new(col, 5.0).draw(
        [a[0] + offset, a[1] + offset, b[0] + offset, b[1] + offset],
        graphics::default_draw_state(),
        graphics::abs_transform(WIDTH as f64, HEIGHT as f64),
        gl
    )

}
