use std::f64::consts::PI;

use babushka::clip::{ClipOp, Clippable};
use babushka::kernelf64::{Point2D, Polygon};
use babushka::multi_polygon::MultiPolygon;
use babushka::point::Point2D as _;
use babushka::polygon::Polygon as _;
use babushka::raster::{draw_multi_polygon, draw_polygon};
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

    let mut union = square.clip_polygon(&triangle, ClipOp::Xor).unwrap();
    let holes = union.split_off(1);
    let outer = union.pop().unwrap();
    let union = MultiPolygon::new(outer, holes);



    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    buffer.fill(0);
    let mut window =
        Window::new("Union", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });




    draw_polygon(&mut buffer, &square, 0xFFFF00, SCALE, WIDTH, HEIGHT);
    draw_polygon(&mut buffer, &triangle, 0xFF00FF, SCALE, WIDTH, HEIGHT);
    draw_multi_polygon(&mut buffer, &union, SCALE, WIDTH, HEIGHT, Some(0xFFFFFF), Some(0x00FFFF));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
