use babushka::kernelf64::{Point2D, Polygon};
use babushka::polygon::Polygon as _;

fn main() {
    let polygon1 = Polygon {
        vertices: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 0.0, y: 1.0 },
            Point2D { x: 1.0, y: 1.0 },
            Point2D { x: 1.0, y: 0.0 },
        ],
        offset: Point2D { x: 0.0, y: 0.0 },
    };

    let mut polygon2 = polygon1.clone();
    polygon2.translate(-2.0, 0.0);
    let direction = Point2D { x: 1.0, y: 0.0 };
    let distance = polygon1.slide_distance_on_polygon(&polygon2, direction, true);
    println!("Slide distance: {:?}", distance);
}
