use babushka::kernelf64::{Point2D, Polygon};
use babushka::multi_polygon::MultiPolygon;
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::point::Point2D as _;
use babushka::polygon::Polygon as _;
use babushka::raster::*;
use std::path::PathBuf;
const FRAME_DELAY: u16 = 20; // 50ms delay between frames

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 1.0;
// const ANIMATION_INTERVAL: Duration = Duration::from_millis(500);

fn main() -> anyhow::Result<()> {
    let n_points = 16;
    let mut outer = Polygon::from((0..n_points).map(|i| {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n_points as f64;
        let x = 100.0 * angle.cos();
        let y = 100.0 * angle.sin();
        Point2D::from_xy(x, y)
    }));
    outer.set_offset(Point2D::from_xy(400.0, 300.0));

    let mut inner = Polygon::from((0..n_points).map(|i| {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n_points as f64;
        let x = 50.0 * angle.cos();
        let y = 50.0 * angle.sin();
        Point2D::from_xy(x, y)
    }));
    inner.set_offset(Point2D::from_xy(400.0, 300.0));

    let piece_0 = MultiPolygon::new(outer, vec![inner]);

    let mut square = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 20.0, y: 0.0 },
        Point2D { x: 20.0, y: 20.0 },
        Point2D { x: 0.0, y: 20.0 },
    ]);
    square.set_offset(Point2D::from_xy(390.0, 290.0));
    // square.set_rotation(PI / 3.0);
    let mut piece_1 = MultiPolygon::new(square, vec![]);

    let mut nfp_list = vec![];
    nfp_list.extend(
        piece_0
            .outer()
            .no_fit_polygon(piece_1.outer(), false, false)
            .unwrap(),
    );
    for hole in piece_0.holes() {
        nfp_list.extend(hole.no_fit_polygon(piece_1.outer(), true, false).unwrap());
    }

    // for v in piece_0.holes().first().unwrap().iter_vertices() {
    //     println!("{{x: {}, y: {}}},", v.x, v.y);
    // }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("assets/nfp_1.gif");

    let nfp_interp: Vec<Vec<Point2D>> = nfp_list.into_iter().map(|x| {
        interpolate_contour(x.into_iter(), 10.0)
    }).collect();
    let num_frames = nfp_interp.iter().map(|x| x.len()).sum();
    println!("num_frames: {}", num_frames);
    create_gif(
        path,
        WIDTH,
        HEIGHT,
        FRAME_DELAY,
        num_frames,
        |frame_index, buffer| {
            draw_multi_polygon(
                buffer,
                &piece_0,
                SCALE,
                WIDTH,
                HEIGHT,
                Some(0xFFFFFF),
                Some(0xFFFF00),
            );
            for contour in &nfp_interp {
                let nfp = Polygon::from(contour.clone());
                draw_polygon(buffer, &nfp, 0xFF0000, SCALE, WIDTH, HEIGHT);
            }
            // let mut piece_1 = piece_1.clone();
            let offset = nfp_interp.iter().flatten().skip(frame_index).next().unwrap();
            piece_1.for_each_polygon(|polygon| {
                polygon.set_offset(*offset);
            });
            draw_multi_polygon(
                buffer,
                &piece_1,
                SCALE,
                WIDTH,
                HEIGHT,
                Some(0xFFFFFF),
                Some(0xFF00FF),
            );

        },
    )
}
