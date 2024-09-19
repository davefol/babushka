use babushka::kernelf64::{Point2D, Polygon};
use babushka::multi_polygon::MultiPolygon;
use babushka::nesting::problem::IrregularBinPackingProblem;
use babushka::parsers::terashima::{parse_terashima, TerashimaInstance};
use babushka::polygon::Polygon as _;
use babushka::raster::{draw_irregular_bin_packing_problem, draw_multi_polygon, draw_polygon, draw_text, TAB10};
use babushka::utils::spread_grid;
use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::path::PathBuf;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 0.07;

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_data/Terashima2/TV001C5.txt");
    let file = File::open(path).unwrap();
    let instance: TerashimaInstance<Polygon> =
        parse_terashima(file).expect("Failed to load Terashima file");
    let problem = IrregularBinPackingProblem::from(instance);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    buffer.fill(0);

    draw_irregular_bin_packing_problem(&mut buffer, &problem, SCALE, WIDTH, HEIGHT, Some(0xFFFFFF), Some(&TAB10));

    let mut window = Window::new("Terashima TV001C5", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });


    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
