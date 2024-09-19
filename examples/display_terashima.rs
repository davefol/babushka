use babushka::kernelf64::{Point2D, Polygon};
use babushka::parsers::terashima::{parse_terashima, TerashimaInstance};
use babushka::multi_polygon::MultiPolygon;
use babushka::polygon::Polygon as _;
use babushka::raster::{draw_multi_polygon, draw_polygon, draw_text};
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
    let mut instance: TerashimaInstance<Polygon> =
        parse_terashima(file).expect("Failed to load Terashima file");

    instance.bin.translate(100.0, 100.0);
    for (idx, location) in spread_grid::<Point2D>(
        instance.pieces.len(),
        WIDTH as f64 / SCALE,
        HEIGHT as f64 / SCALE,
        0.75,
    )
    .enumerate()
    {
        instance.pieces[idx].translate_center_to_point(&location);
    }
    let pieces: Vec<MultiPolygon<_>> = instance.pieces.into_iter().map(|p| MultiPolygon::new(p, vec![])).collect();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    buffer.fill(0);
    let mut window = Window::new("Terashima TV001C5", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Clear the buffer

    // Draw the bin
    draw_polygon(&mut buffer, &instance.bin, 0xFFFFFF, SCALE, WIDTH, HEIGHT);

    // Draw the polygons
    for (i, piece) in pieces.iter().enumerate() {
        let fill_color = 0xFF0000 | ((i as u32 * 50) << 8) | (i as u32 * 30);
        draw_multi_polygon(
            &mut buffer,
            piece,
            SCALE,
            WIDTH,
            HEIGHT,
            Some(0xFFFFFF),
            Some(fill_color),
        );
    }

    // Display information
    draw_text(
        &mut buffer,
        &format!(
            "Bin size: {}x{}",
            instance.bin.bounding_box().max_x,
            instance.bin.bounding_box().max_y
        ),
        10,
        10,
        0xFFFFFF,
        WIDTH,
        HEIGHT,
    );
    draw_text(
        &mut buffer,
        &format!("Number of polygons: {}", pieces.len()),
        10,
        30,
        0xFFFFFF,
        WIDTH,
        HEIGHT,
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
