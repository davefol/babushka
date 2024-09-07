use babushka::kernelf64::Polygon;
use babushka::parsers::terashima::{parse_terashima, TerashimaInstance};
use babushka::polygon::Polygon as _;
use babushka::raster::{best_grid, draw_polygon, draw_text};
use itertools::Itertools;
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

    // move the pieces into a neat grid with some padding
    let (rows, cols) = best_grid(instance.pieces.len(), WIDTH as f64 / HEIGHT as f64);
    for (idx, (i, j)) in (0..rows).cartesian_product(0..cols).enumerate() {
        let x = j as f64 * (WIDTH as f64) * 0.9 / cols as f64 / SCALE;
        let y = i as f64 * (HEIGHT as f64) * 0.9 / rows as f64 / SCALE;
        if idx >= instance.pieces.len() {
            break;
        }
        instance.pieces[idx].set_offset((x, y).into());
        instance.pieces[idx].translate(WIDTH as f64 * 0.1 / SCALE, HEIGHT as f64 * 0.1 / SCALE);
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("Terashima TV001C5", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Clear the buffer
        buffer.fill(0);

        // Draw the bin
        draw_polygon(&mut buffer, &instance.bin, 0xFFFFFF, SCALE, WIDTH, HEIGHT);

        // Draw the polygons
        for (i, polygon) in instance.pieces.iter().enumerate() {
            let color = 0xFF0000 | ((i as u32 * 50) << 8) | (i as u32 * 30);
            draw_polygon(&mut buffer, polygon, color, SCALE, WIDTH, HEIGHT);
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
            &format!("Number of polygons: {}", instance.pieces.len()),
            10,
            30,
            0xFFFFFF,
            WIDTH,
            HEIGHT,
        );

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
