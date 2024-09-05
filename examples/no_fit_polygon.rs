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
        Point2D { x: 3.0, y: 1.0 },
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

    let nfp = polygon1.no_fit_polygon(&polygon2, false, true).unwrap();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    println!("{:?}", nfp);

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
        draw_polygon(&mut buffer, &polygon1, 0xFF0000);
        draw_polygon(&mut buffer, &polygon2, 0x00FF00);
        for nfp_part in &nfp {
            // translate polygon to center
            let mut nfp_polygon = Polygon::from(nfp_part.clone());
            let nfp_bbox = nfp_polygon.bounding_box();
            let center_x = (nfp_bbox.min_x + nfp_bbox.max_x) / 2.0;
            let center_y = (nfp_bbox.min_y + nfp_bbox.max_y) / 2.0;
            nfp_polygon.translate(-center_x, -center_y);
            nfp_polygon.translate(WIDTH as f64 / (2.0 * SCALE), HEIGHT as f64 / (2.0 * SCALE));
            draw_polygon(&mut buffer, &nfp_polygon, 0x0000FF);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn draw_polygon(buffer: &mut Vec<u32>, polygon: &Polygon, color: u32) {
    for segment in polygon.iter_segments() {
        let (x0, y0) = world_to_screen(segment.start.x, segment.start.y);
        let (x1, y1) = world_to_screen(segment.end.x, segment.end.y);
        draw_line(buffer, x0, y0, x1, y1, color);
    }
}

fn draw_polygon_from_points(buffer: &mut Vec<u32>, points: &[Point2D], color: u32) {
    for i in 0..points.len() {
        let start = &points[i];
        let end = &points[(i + 1) % points.len()];
        let (x0, y0) = world_to_screen(start.x, start.y);
        let (x1, y1) = world_to_screen(end.x, end.y);
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
// fn draw_line(buffer: &mut Vec<u32>, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
//     let dx = (x1 - x0).abs();
//     let dy = -(y1 - y0).abs();
//     let sx = if x0 < x1 { 1 } else { -1 };
//     let sy = if y0 < y1 { 1 } else { -1 };
//     let mut err = dx + dy;

//     let mut x = x0;
//     let mut y = y0;

//     loop {
//         if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
//             buffer[y as usize * WIDTH + x as usize] = color;
//         }

//         if x == x1 && y == y1 {
//             break;
//         }

//         let e2 = 2 * err;
//         if e2 >= dy {
//             err += dy;
//             x += sx;
//         }
//         if e2 <= dx {
//             err += dx;
//             y += sy;
//         }
//     }
// }
