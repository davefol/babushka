use babushka::kernelf64::{Point2D, Polygon};
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::polygon::Polygon as _;
use babushka::point::Point2D as _;
use babushka::raster::*;
use minifb::{Key, Window, WindowOptions};
use num_traits::Float;
use std::time::{Duration, Instant};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 50.0;
const ANIMATION_INTERVAL: Duration = Duration::from_millis(500);

fn main() {
    let mut polygon1 = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 2.0, y: 4.0 },
        Point2D { x: 2.0, y: 2.0 },
        Point2D { x: 2.9, y: 1.0 },
        Point2D { x: 5.0, y: 1.0 },
        Point2D { x: 5.0, y: 0.0 },
    ]);
    polygon1.set_rotation(0.0);
    //polygon1.vertices.iter_mut().for_each(|v| *v = v.rotate(0.5));
    polygon1.translate(5.0, 5.0);

    let mut polygon2 = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 1.0, y: 1.0 },
        Point2D { x: 1.0, y: -1.0 },
    ]);
    //polygon2.vertices.iter_mut().for_each(|v| *v = v.rotate(0.9));
    for v in polygon2.vertices.iter_mut() {
        println!("{:?}", v);
    }
    polygon2.set_rotation(0.0);
    polygon2.translate(8.0, 8.0);

    let nfp = polygon1.no_fit_polygon(&polygon2, false, false).unwrap();
    for n in &nfp {
        for v in n {
            println!("{:?}", v);
        }
    }
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

    let mut last_update = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        if now.duration_since(last_update) >= ANIMATION_INTERVAL {
            buffer.fill(0); // Clear the buffer

            draw_polygon(&mut buffer, &polygon1, 0xFF0000, SCALE, WIDTH, HEIGHT);

            // Animate polygon2 by setting its offset to values from the nfp
            if !nfp.is_empty() && !nfp[0].is_empty() {
                polygon2.set_offset(nfp[0][nfp_index]);
                nfp_index = (nfp_index + 1) % nfp[0].len();
            }

            draw_polygon(&mut buffer, &polygon2, 0x00FF00, SCALE, WIDTH, HEIGHT);
            for nfp_part in &nfp {
                let nfp_polygon = Polygon::from(nfp_part.clone());
                draw_polygon(&mut buffer, &nfp_polygon, 0x0000FF, SCALE, WIDTH, HEIGHT);
            }

            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

            last_update = now;
        }

        window.update();
    }
}
