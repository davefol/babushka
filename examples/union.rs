use std::f64::consts::PI;

use babushka::clip::{ClipOp, Clippable};
use babushka::kernelf64::{Point2D, Polygon};
use babushka::point::Point2D as _;
use babushka::polygon::Polygon as _;
use babushka::raster::draw_polygon;
use minifb::{Key, Window, WindowOptions};
const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 10.0;
fn main() {
    let square = Polygon {
        vertices: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 20.0, y: 0.0 },
            Point2D { x: 20.0, y: 20.0 },
            Point2D { x: 0.0, y: 20.0 },
        ],
        offset: Point2D::from_xy(20.0, 20.0),
        rotation: 0.0,
    };

    let triangle = Polygon {
        vertices: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 30.0, y: 0.0 },
            Point2D { x: 15.0, y: 20.0 },
        ],
        offset: Point2D::from_xy(50.0, 15.0),
        rotation: PI / 2.0,
    };

    println!("Subject polygon vertices");
    for point in square.iter_vertices() {
        println!("{:?}", point)
    }

    println!("Clip polygon vertices");
    for point in triangle.iter_vertices() {
        println!("{:?}", point)
    }

    let union = square.clip_polygon(&triangle, ClipOp::Union).unwrap();
    for point in union.iter_vertices() {
        println!("{:?}", point)
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    buffer.fill(0);
    let mut window =
        Window::new("Union", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Clear the buffer

    // Draw the bin
    draw_polygon(&mut buffer, &square, 0xFFFF00, SCALE, WIDTH, HEIGHT);
    draw_polygon(&mut buffer, &triangle, 0xFF00FF, SCALE, WIDTH, HEIGHT);
    draw_polygon(&mut buffer, &union, 0x00FFFF, SCALE, WIDTH, HEIGHT);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
