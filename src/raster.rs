use crate::piece::Piece;
use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::segment::Segment;
use font8x8::UnicodeFonts;
use num_traits::ToPrimitive;
use petgraph::graph::NodeIndex;

pub fn world_to_screen(x: f64, y: f64, scale: f64, height: usize) -> (i32, i32) {
    ((x * scale) as i32, height as i32 - (y * scale) as i32)
}

pub fn draw_polygon<P: Polygon>(
    buffer: &mut Vec<u32>,
    polygon: &P,
    color: u32,
    scale: f64,
    width: usize,
    height: usize,
) {
    for segment in polygon.iter_segments() {
        let (x0, y0) = world_to_screen(
            segment.start().x().to_f64().unwrap(),
            segment.start().y().to_f64().unwrap(),
            scale,
            height,
        );
        let (x1, y1) = world_to_screen(
            segment.end().x().to_f64().unwrap(),
            segment.end().y().to_f64().unwrap(),
            scale,
            height,
        );
        draw_line(buffer, x0, y0, x1, y1, color, width, height);
    }
}

pub fn draw_line(
    buffer: &mut Vec<u32>,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: u32,
    width: usize,
    height: usize,
) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x1;
    let mut y = y1;

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            buffer[y as usize * width + x as usize] = color;
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

pub fn draw_segment<S: Segment>(
    buffer: &mut Vec<u32>,
    segment: &S,
    color: u32,
    scale: f64,
    width: usize,
    height: usize,
) {
    let (x1, y1) = world_to_screen(
        segment.start().x().to_f64().unwrap(),
        segment.start().y().to_f64().unwrap(),
        scale,
        height,
    );
    let (x2, y2) = world_to_screen(
        segment.end().x().to_f64().unwrap(),
        segment.end().y().to_f64().unwrap(),
        scale,
        height,
    );
    draw_line(buffer, x1, y1, x2, y2, color, width, height);
}

pub fn draw_direction<P: Point2D>(
    buffer: &mut Vec<u32>,
    start: &P,
    direction: &P,
    color: u32,
    scale: f64,
    width: usize,
    height: usize,
) {
    let (x1, y1) = world_to_screen(
        start.x().to_f64().unwrap(),
        start.y().to_f64().unwrap(),
        scale,
        height,
    );
    let (x2, y2) = world_to_screen(
        start.x().to_f64().unwrap() + direction.x().to_f64().unwrap(),
        start.y().to_f64().unwrap() + direction.y().to_f64().unwrap(),
        scale,
        height,
    );
    draw_line(buffer, x1, y1, x2, y2, color, width, height);
}

pub fn draw_text(
    buffer: &mut Vec<u32>,
    text: &str,
    x: usize,
    y: usize,
    color: u32,
    width: usize,
    height: usize,
) {
    for (i, c) in text.chars().enumerate() {
        let cx = x + i * 8;
        let cy = y;
        draw_char(buffer, c, cx, cy, color, width, height);
    }
}

pub fn draw_char(
    buffer: &mut Vec<u32>,
    c: char,
    x: usize,
    y: usize,
    color: u32,
    width: usize,
    height: usize,
) {
    let font = font8x8::BASIC_FONTS.get(c).unwrap();
    for (row, byte) in font.iter().enumerate() {
        for col in 0..8 {
            if (byte & (1 << col)) != 0 {
                let px = x + col;
                let py = y + row;
                if px < width && py < height {
                    buffer[py * width + px] = color;
                }
            }
        }
    }
}

pub fn best_grid(n: usize, aspect_ratio: f64) -> (usize, usize) {
    let mut best_rows = 1;
    let mut best_cols = n;
    let mut min_diff = f64::INFINITY;
    for rows in 1..=n {
        let cols = (n as f64 / rows as f64).ceil() as usize;
        let actual_ratio = cols as f64 / rows as f64;
        let diff = (actual_ratio - aspect_ratio).abs();
        if diff < min_diff {
            min_diff = diff;
            best_rows = rows;
            best_cols = cols;
        }
    }
    (best_rows, best_cols)
}

pub fn draw_piece<P: Polygon>(
    buffer: &mut Vec<u32>,
    piece: &Piece<P>,
    root_index: NodeIndex,
    scale: f64,
    width: usize,
    height: usize,
) {
    // Draw the rectangle (outer polygon)
    if let Some(rectangle) = piece.get_polygon(root_index) {
        draw_polygon(buffer, rectangle, 0xFF0000, scale, width, height);
    }

    // Draw the triangular hole (inner polygon)
    for child_index in piece.iter_children(root_index) {
        if let Some(triangle) = piece.get_polygon(child_index) {
            draw_polygon(buffer, triangle, 0x00FF00, scale, width, height);
        }
    }
}