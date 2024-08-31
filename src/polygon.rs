use crate::point::Point2D;
use crate::segment::Segment;
use crate::bounding_box::BoundingBox;
use num_traits::Float;

pub trait Polygon {
    type Point: Point2D;
    type Segment: Segment<Point = Self::Point>;

    fn iter_vertices(&self) -> impl Iterator<Item = &Self::Point>;
    fn iter_mut_vertices(&mut self) -> impl Iterator<Item = &mut Self::Point>;
    fn iter_segments(&self) -> impl Iterator<Item = Self::Segment>;
    fn offset(&self) -> &Self::Point;
    fn length(&self) -> usize;

    fn bounding_box(&self) -> BoundingBox<<<Self as Polygon>::Point as Point2D>::Value> {
        let mut min_x = self.iter_vertices().next().unwrap().x();
        let mut min_y = self.iter_vertices().next().unwrap().y();
        let mut max_x = self.iter_vertices().next().unwrap().x();
        let mut max_y = self.iter_vertices().next().unwrap().y();
        for vertex in self.iter_vertices().skip(1) {
            min_x = min_x.min(vertex.x());
            min_y = min_y.min(vertex.y());
            max_x = max_x.max(vertex.x());
            max_y = max_y.max(vertex.y());
        }
        BoundingBox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    fn translate(&mut self, dx: <<Self as Polygon>::Point as Point2D>::Value, dy: <<Self as Polygon>::Point as Point2D>::Value) {
        for vertex in self.iter_mut_vertices() {
            vertex.set_x(vertex.x() + dx);
            vertex.set_y(vertex.y() + dy);
        }
    }
}