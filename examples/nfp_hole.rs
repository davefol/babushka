use babushka::kernelf64::{Point2D, Polygon};
use babushka::multi_polygon::MultiPolygon;
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::point::Point2D as _;
use babushka::polygon::Polygon as _;
use babushka::raster::*;
use minifb::{Key, Window, WindowOptions};
use std::f64::consts::PI;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 1.0;
// const ANIMATION_INTERVAL: Duration = Duration::from_millis(500);

fn main() {
    let n_points = 16;
    let mut outer = Polygon::from((0..n_points).map(|i| {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n_points as f64;
        let x = 100.0 * angle.cos();
        let y = 100.0 * angle.sin();
        Point2D::from_xy(x, y)
    }));
    for v in outer.iter_mut_vertices_local() {
        v.x += 400.0;
        v.y += 300.0;
    }
    // outer.set_offset(Point2D::from_xy(400.0, 300.0));

    let mut inner = Polygon::from((0..n_points).map(|i| {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n_points as f64;
        let x = 50.0 * angle.cos();
        let y = 50.0 * angle.sin();
        Point2D::from_xy(x, y)
    }));
    for v in inner.iter_mut_vertices_local() {
        v.x += 400.0;
        v.y += 300.0
    }
    // inner.set_offset(Point2D::from_xy(400.0, 300.0));

    let piece_0 = MultiPolygon::new(outer, vec![inner]);

    let mut square = Polygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 20.0, y: 0.0 },
        Point2D { x: 20.0, y: 20.0 },
        Point2D { x: 0.0, y: 20.0 },
    ]);
    for v in square.iter_mut_vertices_local() {
        v.x += 390.0;
        v.y += 290.0;
    }
    // square.set_offset(Point2D::from_xy(390.0, 290.0));
    // square.set_rotation(PI / 3.0);
    let piece_1 = MultiPolygon::new(square, vec![]);

    
    let mut nfp_list = vec![];
    // nfp_list.extend(piece_0.outer().no_fit_polygon(piece_1.outer(), false, false).unwrap());
    for hole in piece_0.holes() {
        nfp_list.extend(hole.no_fit_polygon(piece_1.outer(), true, false).unwrap());
    }
    // for v in piece_0.holes().first().unwrap().iter_vertices() {
    //     println!("{{x: {}, y: {}}},", v.x, v.y);
    // }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "No Fit Polygon with Hole",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    draw_multi_polygon(&mut buffer, &piece_0, SCALE, WIDTH, HEIGHT, Some(0xFFFFFF), Some(0xFFFF00));
    draw_multi_polygon(&mut buffer, &piece_1, SCALE, WIDTH, HEIGHT, Some(0xFFFFFF), Some(0xFF00FF));
    for contour in nfp_list {
        let nfp = Polygon::from(contour);
        draw_polygon(&mut buffer, &nfp, 0xFF0000, SCALE, WIDTH, HEIGHT);
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

}
