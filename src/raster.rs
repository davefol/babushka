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

pub fn screen_to_world(x: i32, y: i32, scale: f64, height: usize) -> (f64, f64) {
    (x as f64 / scale, (height as f64 - y as f64) / scale)
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

/// Draws a filled piece onto a buffer
pub fn draw_piece<P: Polygon>(
    buffer: &mut Vec<u32>,
    piece: &Piece<P>,
    scale: f64,
    width: usize,
    height: usize,
    stroke_color: Option<u32>,
    fill_color: Option<u32>
) where
    <P as Polygon>::Segment: From<(<P as Polygon>::Point, <P as Polygon>::Point)>,
    <P as Polygon>::Point: From<(f64, f64)>,
{
    for node_index in piece.get_roots() {
        draw_piece_node(buffer, piece, *node_index, scale, width, height, stroke_color, fill_color);
    }
}

fn draw_piece_node<P: Polygon>(
    buffer: &mut Vec<u32>,
    piece: &Piece<P>,
    node_index: NodeIndex,
    scale: f64,
    width: usize,
    height: usize,
    stroke_color: Option<u32>,
    fill_color: Option<u32>,
) where
    <P as Polygon>::Segment: From<(<P as Polygon>::Point, <P as Polygon>::Point)>,
    <P as Polygon>::Point: From<(f64, f64)>,
{
    if let Some(fill_color) = fill_color {
        if piece.node_depth(node_index).unwrap() % 2 == 0 {
            let polygon = piece.get_polygon(node_index).unwrap();
            let bounding_box = polygon.bounding_box();
            let (x0, y0) = world_to_screen(
                bounding_box.min_x.to_f64().unwrap(),
                bounding_box.min_y.to_f64().unwrap(),
                scale,
                height,
            );
            let (x1, y1) = world_to_screen(
                bounding_box.max_x.to_f64().unwrap(),
                bounding_box.max_y.to_f64().unwrap(),
                scale,
                height,
            );
            for y_screen in y1..y0 {
                let start: <P as Polygon>::Point =
                    screen_to_world(x0 - 1, y_screen, scale, height).into();
                let end: <P as Polygon>::Point =
                    screen_to_world(x1 + 1, y_screen, scale, height).into();
                let segment = <P as Polygon>::Segment::from((start, end));
                let mut intersections = segment.intersects_polygon(polygon);
                for child_index in piece.iter_children(node_index) {
                    let child_polygon = piece.get_polygon(child_index).unwrap();
                    intersections.extend(segment.intersects_polygon(child_polygon));
                }
                intersections.sort_by(|a, b| a.x().partial_cmp(&b.x()).unwrap());
                let intersections_x: Vec<i32> = intersections
                    .into_iter()
                    .map(|p| {
                        world_to_screen(
                            p.x().to_f64().unwrap(),
                            p.y().to_f64().unwrap(),
                            scale,
                            height,
                        )
                        .0
                    })
                    .collect();

                for pair in intersections_x.chunks_exact(2) {
                    if let [fx0, fx1] = pair {
                        let fx0 = *fx0;
                        let fx1 = *fx1;
                        let fy0 = y_screen;
                        let fy1 = y_screen;
                        draw_line(buffer, fx0, fy0, fx1, fy1, fill_color, width, height);
                    }
                }
            }
        }
    }
    if let Some(stroke_color) = stroke_color {
        let polygon = piece.get_polygon(node_index).unwrap();
        draw_polygon(buffer, polygon, stroke_color, scale, width, height);
    }

    for child_index in piece.iter_children(node_index) {
        draw_piece_node(
            buffer,
            piece,
            child_index,
            scale,
            width,
            height,
            stroke_color,
            fill_color,
        );
    }
}
