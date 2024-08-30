use minifb::{Key, Window, WindowOptions};
use babushka::polygon::Polygon;
use babushka::point::Point2D;

fn main() {
    let mut window = Window::new(
        "Polygons",
        800,
        600,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let polygon1 = Polygon {
        vertices: vec![
            Point2D { x: 100, y: 100 },
            Point2D { x: 200, y: 100 },
            Point2D { x: 150, y: 200 },
        ],
    };
    let polygon2 = Polygon {
        vertices: vec![
            Point2D { x: 400, y: 300 },
            Point2D { x: 500, y: 300 },
            Point2D { x: 450, y: 400 },
            Point2D { x: 350, y: 400 },
        ],
    };

    let mut buffer: Vec<u32> = vec![0; 800 * 600];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        draw_polygon(&mut buffer, &polygon1, 0xFF0000);
        draw_polygon(&mut buffer, &polygon2, 0x00FF00);

        window.update_with_buffer(&buffer, 800, 600).unwrap();
    }
}

fn draw_polygon<T: Copy + Into<i32>>(buffer: &mut Vec<u32>, polygon: &Polygon<T>, color: u32) {
    let vertices = &polygon.vertices;
    for i in 0..vertices.len() {
        let p1 = &vertices[i];
        let p2 = &vertices[(i + 1) % vertices.len()];
        draw_line(buffer, p1.x.into(), p1.y.into(), p2.x.into(), p2.y.into(), color);
    }
}

fn draw_line(buffer: &mut Vec<u32>, mut x1: i32, mut y1: i32, x2: i32, y2: i32, color: u32) {
    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x1 >= 0 && x1 < 800 && y1 >= 0 && y1 < 600 {
            buffer[(y1 * 800 + x1) as usize] = color;
        }
        if x1 == x2 && y1 == y2 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x1 += sx; }
        if e2 <= dx { err += dx; y1 += sy; }
    }
}