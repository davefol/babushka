use crate::point::Point2D;
use crate::segment::Segment;
use crate::{bounding_box::BoundingBox, segment::SegmentSegmentIntersection};
use approx::abs_diff_eq;
use itertools::Itertools;
use num_traits::{Float, One, ToPrimitive, Zero};

pub trait Polygon: Clone + std::fmt::Debug {
    type Point: Point2D;
    type Segment: Segment<Point = Self::Point>;

    /// Returns an in order iterator over the vertices of the polygon.
    /// Coordinates are local to the polygon
    fn iter_vertices_local(&self) -> impl Iterator<Item = &Self::Point>;

    /// Returns an in order iterator over the segments of the polygon.
    /// Coordinates are local to the polygon
    fn iter_segments_local(&self) -> impl Iterator<Item = Self::Segment> + Clone;

    /// Returns the offset of the polygon.
    fn offset(&self) -> Self::Point;

    /// Sets the offset of the polygon.
    fn set_offset(&mut self, offset: Self::Point);

    /// Returns the rotation of the polygon.
    fn rotation(&self) -> <<Self as Polygon>::Point as Point2D>::Value;

    /// Sets the rotation of the polygon.
    fn set_rotation(&mut self, rotation: <<Self as Polygon>::Point as Point2D>::Value);

    /// Returns the number of vertices of the polygon.
    fn length(&self) -> usize;

    /// Returns an inorder iterator over the vertices of the polygon.
    /// Coordinates are after any transformations to the polygon.
    fn iter_vertices(&self) -> impl Iterator<Item = Self::Point> {
        self.iter_vertices_local()
            .map(|vertex| vertex.rotate(self.rotation()) + self.offset())
    }

    /// Returns an inorder iterator over the segments of the polygon.
    /// Coordinates are after any transformations to the polygon.
    fn iter_segments(&self) -> impl Iterator<Item = Self::Segment> + Clone {
        self.iter_segments_local()
            .map(|segment| segment.rotate(self.rotation()) + self.offset())
    }

    fn iter_poly_segments_3(
        &self,
    ) -> impl Iterator<Item = (Self::Segment, Self::Segment, Self::Segment)> + Clone {
        self.iter_segments()
            .zip(self.iter_segments().cycle().skip(1))
            .take(self.length())
            .zip(self.iter_segments().cycle().skip(2))
            .take(self.length())
            .map(|((a, b), c)| (a, b, c))
    }

    /// Returns the local axis aligned bounding box of the polygon.
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

    /// Returns the axis aligned bounding box of the polygon after any transformations.
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

    /// Translates the polygon to the point
    fn translate_to_point(&mut self, point: &Self::Point) {
        let offset = self.offset().clone();
        let dx = point.x() - offset.x();
        let dy = point.y() - offset.y();
        self.translate(dx, dy);
    }

    /// Translates the bounding box center of the polygon to the point
    fn translate_center_to_point(&mut self, point: &Self::Point) {
        let center = self.bounding_box().center();
        self.translate_to_point(&(*point - center));
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

    fn intersects_polygon(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        for ((s00, s01, s02), (s10, s11, s12)) in self
            .iter_poly_segments_3()
            .cartesian_product(other.iter_poly_segments_3())
        {
            let a0 = s00.start();
            let a1 = s01.start();
            let a2 = s01.end();
            let a3 = s02.end();

            let b0 = s10.start();
            let b1 = s11.start();
            let b2 = s11.end();
            let b3 = s12.end();

            if b1.on_segment(&s01) || abs_diff_eq!(a1, b1) {
                let b0in = b0.in_polygon(self).unwrap_or(false);
                let b2in = b2.in_polygon(self).unwrap_or(false);
                if b0in && !b2in || !b0in && b2in {
                    return true;
                } else {
                    continue;
                }
            }

            if b2.on_segment(&s01) || abs_diff_eq!(a2, b2) {
                let b1in = b1.in_polygon(self).unwrap_or(false);
                let b3in = b3.in_polygon(self).unwrap_or(false);
                if b1in && !b3in || !b1in && b3in {
                    return true;
                } else {
                    continue;
                }
            }

            if a1.on_segment(&s11) || abs_diff_eq!(a1, b2) {
                let a0in = a0.in_polygon(other).unwrap_or(false);
                let a2in = a2.in_polygon(other).unwrap_or(false);
                if a0in && !a2in || !a0in && a2in {
                    return true;
                } else {
                    continue;
                }
            }

            if a2.on_segment(&s11) || abs_diff_eq!(a2, b1) {
                let a1in = a1.in_polygon(other).unwrap_or(false);
                let a3in = a3.in_polygon(other).unwrap_or(false);
                if a1in && !a3in || !a1in && a3in {
                    return true;
                } else {
                    continue;
                }
            }

            if let SegmentSegmentIntersection::Intersection(_) = s01.intersects_segment(&s11, false)
            {
                return true;
            }
        }
        false
    }

    fn slide_distance_on_polygon(
        &self,
        other: &Self,
        direction: Self::Point,
        ignore_negative: bool,
    ) -> Option<<<Self as Polygon>::Point as Point2D>::Value> {
        let Some(dir) = direction.normalized() else {
            return None;
        };
        let mut distance = None;
        for (a, b) in self
            .iter_segments()
            .cartesian_product(other.iter_segments())
        {
            // ignore very small segments
            if abs_diff_eq!(a.start(), a.end()) || abs_diff_eq!(b.start(), b.end()) {
                continue;
            }

            let d = a.distance_to_segment_along_direction(&b, dir);

            if let Some(d) = d {
                // if current distance is less than distance then update
                if distance.is_none() || d < distance.unwrap() {
                    if !ignore_negative
                        || d > Zero::zero()
                        || abs_diff_eq!(
                            d,
                            Zero::zero(),
                            epsilon = <<Self as Polygon>::Point as Point2D>::value_epsilon()
                        )
                    {
                        distance = Some(d);
                    }
                }
            }
        }
        distance
    }

    /// project all points of other onto this polygon and return min distance
    fn project_distance_on_polygon(
        &self,
        other: &Self,
        direction: Self::Point,
    ) -> Option<<<Self as Polygon>::Point as Point2D>::Value> {
        let mut distance = None;
        for other_vertex in other.iter_vertices() {
            let mut min_projection = None;
            for self_segment in self.iter_segments() {
				// if (Math.abs((s2.y - s1.y) * direction.x - (s2.x - s1.x) * direction.y) < TOL) {
                if ((self_segment.end().y() - self_segment.start().y()) * direction.x()
                    - (self_segment.end().x() - self_segment.start().x()) * direction.y()).abs()
                    < Self::Point::epsilon()
                {
                    continue;
                }

                // project point, ignore edge boundaries
                let d = other_vertex.distance_to_segment(&self_segment, direction, false);
                if cfg!(debug_assertions) {
                    println!(
                        "\t\t\td: {}, self_segment: ({}, {}) -> ({}, {}), direction: {}, {}",
                        match d {
                            Some(d) => format!("{}", d.to_f64().unwrap()),
                            None => format!("null"),
                        },
                        &self_segment.start().x().to_f64().unwrap(),
                        &self_segment.start().y().to_f64().unwrap(),
                        &self_segment.end().x().to_f64().unwrap(),
                        &self_segment.end().y().to_f64().unwrap(),
                        &direction.x().to_f64().unwrap(),
                        &direction.y().to_f64().unwrap(),
                    )
                }

                if d.is_some() && (min_projection.is_none() || d.unwrap() < min_projection.unwrap())
                {
                    min_projection = d;
                }
            }
            if min_projection.is_some()
                && (distance.is_none() || min_projection.unwrap() > distance.unwrap())
            {
                distance = min_projection
            }
        }

        distance
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
            rotation: 0.0,
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
            rotation: 0.0,
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
            rotation: 0.0,
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
            rotation: 0.0,
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
            rotation: 0.0,
        };

        // Test the area of the counter-clockwise polygon (should be negative)
        assert_eq!(ccw_polygon.area(), -16.0);
    }

    #[test]
    fn test_iter_poly_segments_3() {
        use super::Polygon as _;
        use super::Segment as _;
        use crate::kernelf64::{Point2D, Polygon, Segment};

        let square = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 1.0, y: 0.0 },
                Point2D { x: 1.0, y: 1.0 },
                Point2D { x: 0.0, y: 1.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        let segments: Vec<(Segment, Segment, Segment)> = square.iter_poly_segments_3().collect();

        assert_eq!(segments.len(), 4);

        // Check first triplet
        assert_eq!(segments[0].0.start(), &Point2D { x: 0.0, y: 0.0 });
        assert_eq!(segments[0].0.end(), &Point2D { x: 1.0, y: 0.0 });
        assert_eq!(segments[0].1.start(), &Point2D { x: 1.0, y: 0.0 });
        assert_eq!(segments[0].1.end(), &Point2D { x: 1.0, y: 1.0 });
        assert_eq!(segments[0].2.start(), &Point2D { x: 1.0, y: 1.0 });
        assert_eq!(segments[0].2.end(), &Point2D { x: 0.0, y: 1.0 });

        // Check second triplet
        assert_eq!(segments[1].0.start(), &Point2D { x: 1.0, y: 0.0 });
        assert_eq!(segments[1].0.end(), &Point2D { x: 1.0, y: 1.0 });
        assert_eq!(segments[1].1.start(), &Point2D { x: 1.0, y: 1.0 });
        assert_eq!(segments[1].1.end(), &Point2D { x: 0.0, y: 1.0 });
        assert_eq!(segments[1].2.start(), &Point2D { x: 0.0, y: 1.0 });
        assert_eq!(segments[1].2.end(), &Point2D { x: 0.0, y: 0.0 });

        // Check third triplet
        assert_eq!(segments[2].0.start(), &Point2D { x: 1.0, y: 1.0 });
        assert_eq!(segments[2].0.end(), &Point2D { x: 0.0, y: 1.0 });
        assert_eq!(segments[2].1.start(), &Point2D { x: 0.0, y: 1.0 });
        assert_eq!(segments[2].1.end(), &Point2D { x: 0.0, y: 0.0 });
        assert_eq!(segments[2].2.start(), &Point2D { x: 0.0, y: 0.0 });
        assert_eq!(segments[2].2.end(), &Point2D { x: 1.0, y: 0.0 });

        // Check last triplet
        assert_eq!(segments[3].0.start(), &Point2D { x: 0.0, y: 1.0 });
        assert_eq!(segments[3].0.end(), &Point2D { x: 0.0, y: 0.0 });
        assert_eq!(segments[3].1.start(), &Point2D { x: 0.0, y: 0.0 });
        assert_eq!(segments[3].1.end(), &Point2D { x: 1.0, y: 0.0 });
        assert_eq!(segments[3].2.start(), &Point2D { x: 1.0, y: 0.0 });
        assert_eq!(segments[3].2.end(), &Point2D { x: 1.0, y: 1.0 });
    }

    #[test]
    fn test_polygon_intersects_polygon() {
        use super::Polygon as _;
        use crate::kernelf64::{Point2D, Polygon};

        // Create two intersecting squares
        let square1 = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 2.0, y: 0.0 },
                Point2D { x: 2.0, y: 2.0 },
                Point2D { x: 0.0, y: 2.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        let square2 = Polygon {
            vertices: vec![
                Point2D { x: 1.0, y: 1.0 },
                Point2D { x: 3.0, y: 1.0 },
                Point2D { x: 3.0, y: 3.0 },
                Point2D { x: 1.0, y: 3.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        // Test intersecting polygons
        assert!(square1.intersects_polygon(&square2));
        assert!(square2.intersects_polygon(&square1));

        // Create two non-intersecting squares
        let square3 = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 1.0, y: 0.0 },
                Point2D { x: 1.0, y: 1.0 },
                Point2D { x: 0.0, y: 1.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        let square4 = Polygon {
            vertices: vec![
                Point2D { x: 2.0, y: 2.0 },
                Point2D { x: 3.0, y: 2.0 },
                Point2D { x: 3.0, y: 3.0 },
                Point2D { x: 2.0, y: 3.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        // Test non-intersecting polygons
        assert!(!square3.intersects_polygon(&square4));
        assert!(!square4.intersects_polygon(&square3));
    }

    #[test]
    fn test_polygon_slide_distance_on_polygon() {
        use super::Polygon as _;
        use crate::kernelf64::{Point2D, Polygon};

        // Create two polygons
        let polygon1 = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 0.0, y: 1.0 },
                Point2D { x: 1.0, y: 1.0 },
                Point2D { x: 1.0, y: 0.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };
        let mut polygon2 = polygon1.clone();
        polygon2.translate(-2.0, 0.0);

        // Test slide distance in different directions
        let direction_right = Point2D { x: 1.0, y: 0.0 };
        let distance_right = polygon1.slide_distance_on_polygon(&polygon2, direction_right, true);
        assert!(distance_right.is_some());
        assert_eq!(distance_right.unwrap(), 1.0);

        let direction_left = Point2D { x: -1.0, y: 0.0 };
        let distance_left = polygon2.slide_distance_on_polygon(&polygon1, direction_left, true);
        assert!(distance_left.is_some());
        assert_eq!(distance_left.unwrap(), 1.0);

        let direction_up = Point2D { x: 0.0, y: 1.0 };
        let distance_up = polygon1.slide_distance_on_polygon(&polygon2, direction_up, true);
        assert!(distance_up.is_none());

        let direction_down = Point2D { x: 0.0, y: -1.0 };
        let distance_down = polygon2.slide_distance_on_polygon(&polygon1, direction_down, true);
        assert!(distance_down.is_none());

        let direction_left_ignore = Point2D { x: -1.0, y: 0.0 };
        let distance_left_ignore =
            polygon1.slide_distance_on_polygon(&polygon2, direction_left_ignore, true);
        assert!(distance_left_ignore.is_none());
    }

    #[test]
    fn test_polygon_project_distance_on_polygon() {
        use super::Polygon as _;
        use crate::kernelf64::{Point2D, Polygon};

        // Create two polygons
        let polygon1 = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 0.0, y: 2.0 },
                Point2D { x: 2.0, y: 2.0 },
                Point2D { x: 2.0, y: 0.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };
        let mut polygon2 = polygon1.clone();
        polygon2.translate(3.0, 1.0);

        // Test project distance in different directions
        let direction_left = Point2D { x: -1.0, y: 0.0 };
        let distance_left = polygon1.project_distance_on_polygon(&polygon2, direction_left);
        assert_eq!(distance_left, Some(3.0));

        let direction_right = Point2D { x: 1.0, y: 0.0 };
        let distance_right = polygon2.project_distance_on_polygon(&polygon1, direction_right);
        assert_eq!(distance_right, Some(3.0));

        let direction_up = Point2D { x: 0.0, y: 1.0 };
        let distance_up = polygon1.project_distance_on_polygon(&polygon2, direction_up);
        assert_eq!(distance_up, None);

        let direction_down = Point2D { x: 0.0, y: -1.0 };
        let distance_down = polygon2.project_distance_on_polygon(&polygon1, direction_down);
        assert_eq!(distance_down, None);
    }
}
