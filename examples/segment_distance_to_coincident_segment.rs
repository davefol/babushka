use babushka::kernelf64::{Point2D, Segment};
use babushka::raster::*;
use babushka::segment::Segment as _;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 50.0;
const TRANSLATION_SPEED: f64 = 0.01;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Segment Distance to Coincident Segment",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let segment1 = Segment {
        start: Point2D { x: 2.0, y: 2.0 },
        end: Point2D { x: 2.0, y: 6.0 },
    };

    let mut segment2 = Segment {
        start: Point2D {
            x: 4.0,
            y: 2.0 + 3.0,
        },
        end: Point2D {
            x: 4.0,
            y: 6.0 + 3.0,
        },
    };

    let direction = Point2D { x: -1.0, y: 0.0 };
    let mut translation = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        draw_segment(&mut buffer, &segment1, 0xFF0000, SCALE, WIDTH, HEIGHT);
        draw_segment(&mut buffer, &segment2, 0x00FF00, SCALE, WIDTH, HEIGHT);
        draw_direction(
            &mut buffer,
            &segment1.start,
            &direction,
            0x0000FF,
            SCALE,
            WIDTH,
            HEIGHT,
        );

        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        let distance_text = match distance {
            Some(d) => format!("Distance: {:.2}", d),
            None => "No valid distance".to_string(),
        };

        draw_text(&mut buffer, &distance_text, 10, 10, 0xFFFFFF, WIDTH, HEIGHT);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        buffer.fill(0);

        // Translate segment2 horizontally
        translation += TRANSLATION_SPEED;
        segment2.start.x = 2.0 + translation.sin() * 2.0;
        segment2.end.x = 2.0 + translation.sin() * 2.0;
    }
}
