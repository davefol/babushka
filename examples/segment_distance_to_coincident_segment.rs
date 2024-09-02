use babushka::kernelf64::{Point2D, Segment};
use babushka::segment::Segment as _;
use babushka::point::Point2D as _;
use font8x8::BASIC_FONTS;
use font8x8::UnicodeFonts;
use minifb::{Key, Window, WindowOptions};
use std::f64::consts::PI;

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
        start: Point2D { x: 4.0, y: 2.0 + 3.0 },
        end: Point2D { x: 4.0, y: 6.0 + 3.0 },
    };

    let direction = Point2D { x: -1.0, y: 0.0 };
    let mut translation = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        draw_segment(&mut buffer, &segment1, 0xFF0000);
        draw_segment(&mut buffer, &segment2, 0x00FF00);
        draw_direction(&mut buffer, &segment1.start, &direction, 0x0000FF);

        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        let distance_text = match distance {
            Some(d) => format!("Distance: {:.2}", d),
            None => "No valid distance".to_string(),
        };

        draw_text(&mut buffer, &distance_text, 10, 10, 0xFFFFFF);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        buffer.fill(0);

        // Translate segment2 horizontally
        translation += TRANSLATION_SPEED;
        segment2.start.x = 4.0 + translation.sin() * 2.0;
        segment2.end.x = 4.0 + translation.sin() * 2.0;
    }
}

fn draw_segment(buffer: &mut Vec<u32>, segment: &Segment, color: u32) {
    let (x1, y1) = world_to_screen(segment.start.x(), segment.start.y());
    let (x2, y2) = world_to_screen(segment.end.x(), segment.end.y());
    draw_line(buffer, x1, y1, x2, y2, color);
}

fn draw_direction(buffer: &mut Vec<u32>, start: &Point2D, direction: &Point2D, color: u32) {
    let (x1, y1) = world_to_screen(start.x(), start.y());
    let (x2, y2) = world_to_screen(start.x() + direction.x(), start.y() + direction.y());
    draw_line(buffer, x1, y1, x2, y2, color);
}

fn world_to_screen(x: f64, y: f64) -> (i32, i32) {
    ((x * SCALE) as i32, HEIGHT as i32 - (y * SCALE) as i32)
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

fn draw_text(buffer: &mut Vec<u32>, text: &str, x: usize, y: usize, color: u32) {
    for (i, c) in text.chars().enumerate() {
        let cx = x + i * 8;
        let cy = y;
        draw_char(buffer, c, cx, cy, color);
    }
}

fn draw_char(buffer: &mut Vec<u32>, c: char, x: usize, y: usize, color: u32) {
    let font = font8x8::BASIC_FONTS.get(c).unwrap();
    for (row, byte) in font.iter().enumerate() {
        for col in 0..8 {
            if (byte & (1 << col)) != 0 {
                let px = x + col;
                let py = y + row;
                if px < WIDTH && py < HEIGHT {
                    buffer[py * WIDTH + px] = color;
                }
            }
        }
    }
}
