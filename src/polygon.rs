use crate::bounding_box::BoundingBox;
use crate::point::Point2D;
use crate::segment::Segment;
use num_traits::{Float, One};

pub trait Polygon {
    type Point: Point2D;
    type Segment: Segment<Point = Self::Point>;

    /// Returns an in order iterator over the vertices of the polygon.
    /// Coordinates are local to the polygon
    fn iter_vertices_local(&self) -> impl Iterator<Item = &Self::Point>;

    /// Returns an in order iterator over mutable vertices of the polygon.
    fn iter_mut_vertices_local(&mut self) -> impl Iterator<Item = &mut Self::Point>;

    /// Returns an in order iterator over the segments of the polygon.
    /// Coordinates are local to the polygon
    fn iter_segments_local(&self) -> impl Iterator<Item = Self::Segment>;

    /// Returns the offset of the polygon.
    fn offset(&self) -> &Self::Point;

    /// Sets the offset of the polygon.
    fn set_offset(&mut self, offset: Self::Point);

    /// Returns the number of vertices of the polygon.
    fn length(&self) -> usize;

    /// Returns an inorder iterator over the vertices of the polygon.
    /// Coordinates are after any transformations to the polygon.
    fn iter_vertices(&self) -> impl Iterator<Item = Self::Point> {
        self.iter_vertices_local()
            .map(|vertex| *vertex + *self.offset())
    }

    /// Returns the local bounding box of the polygon.
    fn bounding_box_local(&self) -> BoundingBox<<<Self as Polygon>::Point as Point2D>::Value> {
        let mut min_x = self.iter_vertices_local().next().unwrap().x();
        let mut min_y = self.iter_vertices_local().next().unwrap().y();
        let mut max_x = self.iter_vertices_local().next().unwrap().x();
        let mut max_y = self.iter_vertices_local().next().unwrap().y();
        for vertex in self.iter_vertices_local().skip(1) {
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

    /// Returns the bounding box of the polygon after any transformations.
    fn bounding_box(&self) -> BoundingBox<<<Self as Polygon>::Point as Point2D>::Value> {
        let bounding_box_local = self.bounding_box_local();
        let min_x = bounding_box_local.min_x + self.offset().x();
        let min_y = bounding_box_local.min_y + self.offset().y();
        let max_x = bounding_box_local.max_x + self.offset().x();
        let max_y = bounding_box_local.max_y + self.offset().y();
        BoundingBox {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Translates the vertices of the polygon before any transformations.
    fn translate_local(
        &mut self,
        dx: <<Self as Polygon>::Point as Point2D>::Value,
        dy: <<Self as Polygon>::Point as Point2D>::Value,
    ) {
        for vertex in self.iter_mut_vertices_local() {
            vertex.set_x(vertex.x() + dx);
            vertex.set_y(vertex.y() + dy);
        }
    }

    /// Translates the vertices of the polygon.
    fn translate(
        &mut self,
        dx: <<Self as Polygon>::Point as Point2D>::Value,
        dy: <<Self as Polygon>::Point as Point2D>::Value,
    ) {
        let mut offset = self.offset().clone();
        offset.set_x(offset.x() + dx);
        offset.set_y(offset.y() + dy);
        self.set_offset(offset);
    }

    /// Return the area of the polygon assuming no self intersections.
    /// A negative area indicates counter-clockwise winding.
    fn area(&self) -> <<Self as Polygon>::Point as Point2D>::Value {
        let two = <<Self as Polygon>::Point as Point2D>::Value::one()
            + <<Self as Polygon>::Point as Point2D>::Value::one();
        self.iter_segments_local()
            .map(|segment| {
                let start = segment.start();
                let end = segment.end();
                (start.x() + end.x()) * (start.y() - end.y())
            })
            .sum::<<<Self as Polygon>::Point as Point2D>::Value>()
            / two
    }
}

mod tests {
    #[test]
    fn test_bounding_box() {
        use super::Polygon as _;
        use crate::kernelf64::{Point2D, Polygon};

        let square = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: 4.0 },
                Point2D { x: 0.0, y: 4.0 },
            ],
            offset: Point2D { x: 1.0, y: 1.0 },
        };

        let bbox = square.bounding_box();
        assert_eq!(bbox.min_x, 1.0);
        assert_eq!(bbox.min_y, 1.0);
        assert_eq!(bbox.max_x, 5.0);
        assert_eq!(bbox.max_y, 5.0);

        let triangle = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 3.0, y: 0.0 },
                Point2D { x: 1.5, y: 2.0 },
            ],
            offset: Point2D { x: -1.0, y: -1.0 },
        };

        let bbox = triangle.bounding_box();
        assert_eq!(bbox.min_x, -1.0);
        assert_eq!(bbox.min_y, -1.0);
        assert_eq!(bbox.max_x, 2.0);
        assert_eq!(bbox.max_y, 1.0);
    }
    #[test]
    fn test_area() {
        use super::Polygon as _;
        use crate::kernelf64::{Point2D, Polygon};

        // Create a square polygon
        let square = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: -4.0 },
                Point2D { x: 0.0, y: -4.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
        };

        // Test the area of the square
        assert_eq!(square.area(), 16.0);

        // Create a triangle polygon
        let triangle = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 2.0, y: -4.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
        };

        // Test the area of the triangle
        assert_eq!(triangle.area(), 8.0);

        // Create a polygon with counter-clockwise winding
        let ccw_polygon = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: 4.0 },
                Point2D { x: 0.0, y: 4.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
        };

        // Test the area of the counter-clockwise polygon (should be negative)
        assert_eq!(ccw_polygon.area(), -16.0);
    }
}
