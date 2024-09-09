use babushka::kernelf64::{Point2D, Segment};
use babushka::point::Point2D as _;
use babushka::raster::{draw_line, draw_segment, world_to_screen};
use babushka::segment::{Segment as SegmentTrait, SegmentSegmentIntersection};
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 100.0;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Segment Intersection",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Define two segments
    let segment1 = Segment {
        start: Point2D { x: 1.0, y: 1.0 },
        end: Point2D { x: 5.0, y: 5.0 },
    };

    let segment2 = Segment {
        start: Point2D { x: 2.0, y: 2.0 },
        end: Point2D { x: 4.0, y: 4.0 },
    };

    // Calculate intersection
    let intersection = segment1.intersects_segment(&segment2, false);

    println!("Intersection: {:?}", intersection);

    // Clear the buffer
    buffer.fill(0);

    // Draw segments
    draw_segment(&mut buffer, &segment1, 0xFF0000, SCALE, WIDTH, HEIGHT);
    draw_segment(&mut buffer, &segment2, 0x00FF00, SCALE, WIDTH, HEIGHT);

    // Draw intersection point if it exists
    match intersection {
        SegmentSegmentIntersection::Intersection(point) => {
            let (x, y) = world_to_screen(point.x(), point.y(), SCALE, HEIGHT);
            draw_line(&mut buffer, x - 5, y, x + 5, y, 0xFFFFFF, WIDTH, HEIGHT);
            draw_line(&mut buffer, x, y - 5, x, y + 5, 0xFFFFFF, WIDTH, HEIGHT);
        }
        SegmentSegmentIntersection::Touching(point) => {
            let (x, y) = world_to_screen(point.x(), point.y(), SCALE, HEIGHT);
            draw_line(&mut buffer, x - 5, y, x + 5, y, 0xFFFFFF, WIDTH, HEIGHT);
            draw_line(&mut buffer, x, y - 5, x, y + 5, 0xFFFFFF, WIDTH, HEIGHT);
        }
        SegmentSegmentIntersection::Overlap(point1, point2) => {
            let (x, y) = world_to_screen(point1.x(), point1.y(), SCALE, HEIGHT);
            draw_line(&mut buffer, x - 5, y, x + 5, y, 0xFFFFFF, WIDTH, HEIGHT);
            draw_line(&mut buffer, x, y - 5, x, y + 5, 0xFFFFFF, WIDTH, HEIGHT);
            let (x, y) = world_to_screen(point2.x(), point2.y(), SCALE, HEIGHT);
            draw_line(&mut buffer, x - 5, y, x + 5, y, 0xFFFFFF, WIDTH, HEIGHT);
            draw_line(&mut buffer, x, y - 5, x, y + 5, 0xFFFFFF, WIDTH, HEIGHT);
        }
        _ => {}
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update the window
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
