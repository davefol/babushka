use std::path::PathBuf;

use crate::multi_polygon::MultiPolygon;
use crate::point::Point2D;
use crate::polygon::Polygon;
use crate::polygon_graph::PolygonGraph;
use crate::segment::{Segment, SegmentSegmentIntersection};
use anyhow::Result;
use approx::abs_diff_eq;
use font8x8::UnicodeFonts;
use gif::{Encoder, Frame, Repeat};
use itertools::Itertools;
use num_traits::{Float, NumCast, One, ToPrimitive};
use petgraph::graph::NodeIndex;
use std::fs::File;

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
pub fn draw_polygon_graph<P: Polygon>(
    buffer: &mut Vec<u32>,
    piece: &PolygonGraph<P>,
    scale: f64,
    width: usize,
    height: usize,
    stroke_color: Option<u32>,
    fill_color: Option<u32>,
) where
    <P as Polygon>::Segment: From<(<P as Polygon>::Point, <P as Polygon>::Point)>,
    <P as Polygon>::Point: From<(f64, f64)>,
{
    for node_index in piece.get_roots() {
        draw_polygon_graph_node(
            buffer,
            piece,
            *node_index,
            scale,
            width,
            height,
            stroke_color,
            fill_color,
        );
    }
}

fn draw_polygon_graph_node<P: Polygon>(
    buffer: &mut Vec<u32>,
    piece: &PolygonGraph<P>,
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
        draw_polygon_graph_node(
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

pub fn draw_multi_polygon<P: Polygon>(
    buffer: &mut Vec<u32>,
    multi_polygon: &MultiPolygon<P>,
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
        let polygon = multi_polygon.outer();
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
            let mut intersections = vec![];
            let start: <P as Polygon>::Point =
                screen_to_world(x0 - 1, y_screen, scale, height).into();
            let end: <P as Polygon>::Point =
                screen_to_world(x1 + 1, y_screen, scale, height).into();

            let scanline = <P as Polygon>::Segment::from((start, end));
            for edge in polygon.iter_segments() {
                match scanline.intersects_segment(&edge, false) {
                    SegmentSegmentIntersection::Equal => {}
                    SegmentSegmentIntersection::None => {}
                    SegmentSegmentIntersection::Overlap(_, _) => {}
                    SegmentSegmentIntersection::Touching(p) => {
                        let above = if abs_diff_eq!(p, edge.start()) {
                            p.y() > edge.end().y()
                        } else {
                            p.y() > edge.start().y()
                        };
                        intersections.push((p, above));
                    }
                    SegmentSegmentIntersection::Intersection(p) => {
                        let above = if abs_diff_eq!(p, edge.start()) {
                            p.y() > edge.end().y()
                        } else {
                            p.y() > edge.start().y()
                        };
                        intersections.push((p, above));
                    }
                }
            }

            for hole_polygon in multi_polygon.holes() {
                for edge in hole_polygon.iter_segments() {
                    match scanline.intersects_segment(&edge, false) {
                        SegmentSegmentIntersection::Equal => {}
                        SegmentSegmentIntersection::None => {}
                        SegmentSegmentIntersection::Overlap(_, _) => {}
                        SegmentSegmentIntersection::Touching(p) => {
                            let above = if abs_diff_eq!(p, edge.start()) {
                                p.y() > edge.end().y()
                            } else {
                                p.y() > edge.start().y()
                            };
                            intersections.push((p, above));
                        }
                        SegmentSegmentIntersection::Intersection(p) => {
                            let above = if abs_diff_eq!(p, edge.start()) {
                                p.y() > edge.end().y()
                            } else {
                                p.y() > edge.start().y()
                            };
                            intersections.push((p, above));
                        }
                    }
                }
            }
            intersections.sort_by(|a, b| a.0.x().partial_cmp(&b.0.x()).unwrap());
            if intersections.len() % 2 != 0 {
                println!("intersections: {:?}", intersections);
            }
            intersections.dedup_by(|a, b| abs_diff_eq!(a.0, b.0) && (a.1 != b.1));
            let intersections_x: Vec<i32> = intersections
                .iter()
                .map(|p| {
                    world_to_screen(
                        p.0.x().to_f64().unwrap(),
                        p.0.y().to_f64().unwrap(),
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
    if let Some(stroke_color) = stroke_color {
        let polygon = multi_polygon.outer();
        draw_polygon(buffer, polygon, stroke_color, scale, width, height);

        for polygon in multi_polygon.holes() {
            draw_polygon(buffer, polygon, stroke_color, scale, width, height);
        }
    }
}

pub fn create_gif<F>(
    output_path: PathBuf,
    width: usize,
    height: usize,
    frame_delay: u16,
    num_frames: usize,
    mut frame_builder: F,
) -> Result<()>
where
    F: FnMut(usize, &mut Vec<u32>) -> (),
{
    // Create the output file
    let mut image = File::create(output_path)?;

    // Initialize the GIF encoder
    let mut encoder = Encoder::new(&mut image, width as u16, height as u16, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    // Initialize the buffer (RGBA)
    let mut buffer: Vec<u32> = vec![0; width * height];

    for frame_index in 0..num_frames {
        buffer.fill(0); // Clear the buffer

        // Let the user-defined closure build the frame
        frame_builder(frame_index, &mut buffer);

        // Convert buffer to RGB format
        let mut frame_buffer = vec![0u8; width * height * 3];
        for (i, pixel) in buffer.iter().enumerate() {
            frame_buffer[i * 3] = ((pixel >> 16) & 0xFF) as u8; // Red
            frame_buffer[i * 3 + 1] = ((pixel >> 8) & 0xFF) as u8; // Green
            frame_buffer[i * 3 + 2] = (pixel & 0xFF) as u8; // Blue
        }

        // Create and write the frame
        let mut frame = Frame::from_rgb(width as u16, height as u16, &frame_buffer);
        frame.delay = frame_delay;
        encoder.write_frame(&frame)?;

        println!("Frame {} generated", frame_index + 1);
    }

    println!("GIF animation saved successfully.");
    Ok(())
}

pub fn interpolate_contour<I, P>(contour: I, interval: P::Value) -> Vec<P>
where
    I: IntoIterator<Item = P>,
    P: Point2D,
{
    contour
        .into_iter()
        .tuple_windows()
        .map(|(a, b)| {
            let slope = b - a;
            let magnitude = slope.dot(&slope).sqrt();
            let n = magnitude / interval;
            let increment = <P as Point2D>::Value::one() / n;

            (0..n.round().to_usize().unwrap()).map(move |x| {
                let scale = <<P as Point2D>::Value as NumCast>::from(x).unwrap() * increment;
                a + slope * scale
            })
        })
        .flatten()
        .collect()
}
