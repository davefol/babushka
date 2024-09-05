use babushka::kernelf64::{Point2D, Polygon};
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::polygon::Polygon as _;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 50.0;

fn world_to_screen(x: f64, y: f64) -> (i32, i32) {
    ((x * SCALE) as i32, HEIGHT as i32 - (y * SCALE) as i32)
}
fn main() {
    let mut polygon1 = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 2.0, y: 4.0 },
        Point2D { x: 2.0, y: 2.0 },
        Point2D { x: 2.9, y: 1.0 },
        Point2D { x: 5.0, y: 1.0 },
        Point2D { x: 5.0, y: 0.0 },
    ]);
    polygon1.translate(5.0, 5.0);

    let mut polygon2 = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 1.0, y: 1.0 },
        Point2D { x: 1.0, y: -1.0 },
    ]);
    polygon2.translate(8.0, 8.0);

    let nfp = polygon1.no_fit_polygon(&polygon2, false, false).unwrap();
    println!("{:?}", nfp);
    let mut nfp_index = 0;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "No Fit Polygon Example",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0); // Clear the buffer

        draw_polygon(&mut buffer, &polygon1, 0xFF0000);

        // Animate polygon2 by setting its offset to values from the nfp
        if !nfp.is_empty() && !nfp[0].is_empty() {
            polygon2.set_offset(nfp[0][nfp_index]);
            nfp_index = (nfp_index + 1) % nfp[0].len();
        }

        draw_polygon(&mut buffer, &polygon2, 0x00FF00);
        for nfp_part in &nfp {
            let nfp_polygon = Polygon::from(nfp_part.clone());
            draw_polygon(&mut buffer, &nfp_polygon, 0x0000FF);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        // Add a small delay to control animation speed
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn draw_polygon(buffer: &mut Vec<u32>, polygon: &Polygon, color: u32) {
    for segment in polygon.iter_segments() {
        let (x0, y0) = world_to_screen(segment.start.x, segment.start.y);
        let (x1, y1) = world_to_screen(segment.end.x, segment.end.y);
        draw_line(buffer, x0, y0, x1, y1, color);
    }
}

fn draw_line(buffer: &mut Vec<u32>, x1: i32, y1: i32, x2: i32, y2: i32, color: u32) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x1;
    let mut y = y1;

    loop {
        if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
            buffer[y as usize * WIDTH + x as usize] = color;
        }

        if x == x2 && y == y2 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}
