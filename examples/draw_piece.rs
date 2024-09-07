use babushka::kernelf64::{Point2D, Polygon as KernelPolygon};
use babushka::piece::Piece;
use babushka::polygon::Polygon;
use babushka::raster::draw_piece;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const SCALE: f64 = 100.0;

fn main() {
    // Create a rectangle
    let rectangle = KernelPolygon::from(vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 4.0, y: 0.0 },
        Point2D { x: 4.0, y: 3.0 },
        Point2D { x: 0.0, y: 3.0 },
    ]);

    // Create a triangular hole
    let triangle = KernelPolygon::from(vec![
        Point2D { x: 1.0, y: 1.0 },
        Point2D { x: 3.0, y: 1.0 },
        Point2D { x: 2.0, y: 2.0 },
    ]);

    // Create a piece with the rectangle as the root
    let mut piece = Piece::new(rectangle);

    // Add the triangular hole as a child of the rectangle
    let rectangle_index = piece.get_roots()[0];
    piece.add_child(rectangle_index, triangle);

    piece.for_each_polygon(|p| p.translate(2.0, 1.5));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Piece with Triangular Hole",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    buffer.fill(0);
    draw_piece(&mut buffer, &piece, SCALE, WIDTH, HEIGHT, 0xFFFFFF);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
