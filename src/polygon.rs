use crate::{bounding_box::BoundingBox, point::Point2D};

#[derive(Debug)]
pub struct Polygon<T> {
    pub vertices: Vec<Point2D<T>>,
}

impl<T: Ord + Copy> Polygon<T> {
    pub fn bounding_box(&self) -> BoundingBox<T> {
        let mut min_x = self.vertices[0].x;
        let mut min_y = self.vertices[0].y;
        let mut max_x = self.vertices[0].x;
        let mut max_y = self.vertices[0].y;

        for vertex in &self.vertices[1..] {
            min_x = min_x.min(vertex.x);
            min_y = min_y.min(vertex.y);
            max_x = max_x.max(vertex.x);
            max_y = max_y.max(vertex.y);
        }

        BoundingBox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn translate(&mut self, dx: T, dy: T)
    where
        T: std::ops::Add<Output = T>,
    {
        for vertex in &mut self.vertices {
            vertex.x = vertex.x + dx;
            vertex.y = vertex.y + dy;
        }
    }
}
