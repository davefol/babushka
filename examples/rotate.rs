use babushka::kernelf64::{Point2D, Polygon};
use babushka::polygon::Polygon as _;
use babushka::raster::{draw_polygon, draw_text};
use minifb::{Key, Window, WindowOptions};
use std::f64::consts::PI;
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 0.5;
const ROTATION_SPEED: f64 = 2.0;

fn main() {
    let mut polygon = Polygon {
        vertices: vec![
            Point2D { x: -50.0, y: -50.0 },
            Point2D { x: 50.0, y: -50.0 },
            Point2D { x: 50.0, y: 50.0 },
            Point2D { x: -50.0, y: 50.0 },
        ],
        offset: Point2D { x: WIDTH as f64 / 2.0 / SCALE, y: HEIGHT as f64 / 2.0 / SCALE },
        rotation: 0.0,
    };

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Rotating Polygon",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let start_time = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0);

        let elapsed = start_time.elapsed();
        let angle = (elapsed.as_secs_f64() * ROTATION_SPEED % (2.0 * PI)) as f64;
        polygon.set_rotation(angle);
        draw_polygon(&mut buffer, &polygon, 0xFFFFFF, SCALE, WIDTH, HEIGHT);

        draw_text(
            &mut buffer,
            &format!("Angle: {:.2}", angle.to_degrees()),
            10,
            10,
            0xFFFFFF,
            WIDTH,
            HEIGHT,
        );

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}