use babushka::kernelf64::{Point2D, Polygon};
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::polygon::Polygon as _;
use babushka::raster::*;
use std::path::PathBuf;
use anyhow::Result;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 50.0;
const FRAME_DELAY: u16 = 50; // 50ms delay between frames

fn main() -> Result<()> {
    let mut polygon1 = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 2.0, y: 4.0 },
        Point2D { x: 2.0, y: 2.0 },
        Point2D { x: 2.9, y: 1.0 },
        Point2D { x: 5.0, y: 1.0 },
        Point2D { x: 5.0, y: 0.0 },
    ]);
    polygon1.set_rotation(0.0);
    polygon1.translate(5.0, 5.0);

    let mut polygon2 = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 1.0, y: 1.0 },
        Point2D { x: 1.0, y: -1.0 },
    ]);
    polygon2.set_rotation(0.0);
    polygon2.translate(8.0, 8.0);

    let nfp = polygon1.no_fit_polygon(&polygon2, false, false).unwrap();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("assets/nfp_0.gif");

    create_gif(
        path,
        WIDTH,
        HEIGHT,
        FRAME_DELAY,
        nfp[0].len(),
        |frame_index, buffer| {
            // Draw the static polygon1 in red
            draw_polygon(buffer, &polygon1, 0xFF0000, SCALE, WIDTH, HEIGHT);

            // Update the position of polygon2 based on NFP points
            polygon2.set_offset(nfp[0][frame_index]);
            // Draw the moving polygon2 in green
            draw_polygon(buffer, &polygon2, 0x00FF00, SCALE, WIDTH, HEIGHT);

            // Draw all NFP polygons in blue
            for nfp_part in &nfp {
                let nfp_polygon = Polygon::from(nfp_part.clone());
                draw_polygon(buffer, &nfp_polygon, 0x0000FF, SCALE, WIDTH, HEIGHT);
            }
        },
    )
}
