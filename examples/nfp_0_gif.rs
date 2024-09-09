use babushka::kernelf64::{Point2D, Polygon};
use babushka::no_fit_polygon::ComputeNoFitPolygon;
use babushka::polygon::Polygon as _;
use babushka::raster::*;
use gif::{Encoder, Frame, Repeat};
use std::fs::File;
use std::path::PathBuf;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 50.0;
const FRAME_DELAY: u16 = 50; // 50ms delay between frames

fn main() -> Result<(), std::io::Error> {
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
    let mut image = File::create(path)?;
    let mut encoder = Encoder::new(&mut image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    for (frame_index, nfp_point) in nfp[0].iter().enumerate() {
        buffer.fill(0); // Clear the buffer

        draw_polygon(&mut buffer, &polygon1, 0xFF0000, SCALE, WIDTH, HEIGHT);

        polygon2.set_offset(*nfp_point);
        draw_polygon(&mut buffer, &polygon2, 0x00FF00, SCALE, WIDTH, HEIGHT);

        for nfp_part in &nfp {
            let nfp_polygon = Polygon::from(nfp_part.clone());
            draw_polygon(&mut buffer, &nfp_polygon, 0x0000FF, SCALE, WIDTH, HEIGHT);
        }

        let mut frame_buffer = vec![0u8; WIDTH * HEIGHT * 3];
        for (i, pixel) in buffer.iter().enumerate() {
            frame_buffer[i * 3] = ((pixel >> 16) & 0xFF) as u8; // Red
            frame_buffer[i * 3 + 1] = ((pixel >> 8) & 0xFF) as u8; // Green
            frame_buffer[i * 3 + 2] = (pixel & 0xFF) as u8; // Blue
        }

        let mut frame = Frame::from_rgb(WIDTH as u16, HEIGHT as u16, &frame_buffer);
        frame.delay = FRAME_DELAY;
        encoder.write_frame(&frame).unwrap();

        println!("Frame {} generated", frame_index + 1);
    }

    println!("GIF animation saved as nfp_animation.gif");
    Ok(())
}
