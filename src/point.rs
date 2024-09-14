use approx::{abs_diff_eq, AbsDiffEq};
use num_traits::{Float, One, Zero};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use crate::polygon::Polygon;
use crate::segment::Segment;

pub trait Point2D:
    Clone
    + Copy
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Neg<Output = Self>
    + Div<Self::Value, Output = Self>
    + Mul<Self::Value, Output = Self>
    + AddAssign<Self::Value>
    + Zero
    + AbsDiffEq
    + std::fmt::Debug
{
    type Value: Float + AbsDiffEq + Copy + Sum + std::fmt::Debug + AddAssign;

    fn x(&self) -> Self::Value;
    fn y(&self) -> Self::Value;
    fn from_xy(x: Self::Value, y: Self::Value) -> Self;

    fn set_x(&mut self, x: Self::Value);
    fn set_y(&mut self, y: Self::Value);

    fn epsilon() -> Self::Value;
    fn value_epsilon() -> <Self::Value as AbsDiffEq>::Epsilon;

    fn dot(&self, other: &Self) -> Self::Value {
        self.x() * other.x() + self.y() * other.y()
    }

    fn rotate(&self, angle: Self::Value) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::from_xy(
            self.x() * cos - self.y() * sin,
            self.x() * sin + self.y() * cos,
        )
    }

    fn translate(&self, other: &Self) -> Self {
        Self::from_xy(self.x() + other.x(), self.y() + other.y())
    }

    /// Returns true if the point is within the given distance of the other point.
    fn within_distance(&self, other: &Self, distance: Self::Value) -> bool {
        let dx = self.x() - other.x();
        let dy = self.y() - other.y();
        (dx * dx + dy * dy) < distance * distance
    }

    /// returns a new point that is the normalized version of this point.
    fn normalized(&self) -> Option<Self>
    where
        Self: Sized,
    {
        if abs_diff_eq!(
            self.x() * self.x() + self.y() * self.y(),
            Self::Value::one(),
            epsilon = Self::value_epsilon()
        ) {
            return Some(self.clone());
        }
        let len = (self.x() * self.x() + self.y() * self.y()).sqrt();
        if len == Self::Value::zero() {
            return None;
        }
        let inverse = Self::Value::one() / len;
        let mut out = self.clone();
        out.set_x(out.x() * inverse);
        out.set_y(out.y() * inverse);
        Some(out)
    }

    /// Returns true if the point is on the line segment between a and b.
    /// not including a or b.
    fn on_segment<T: Segment<Point = Self>>(&self, segment: &T) -> bool {
        let a = segment.start();
        let b = segment.end();
        // Vertical line
        if abs_diff_eq!(a.x(), b.x(), epsilon = Self::value_epsilon())
            && abs_diff_eq!(self.x(), a.x(), epsilon = Self::value_epsilon())
        {
            if !abs_diff_eq!(self.y(), b.y(), epsilon = Self::value_epsilon())
                && !abs_diff_eq!(self.y(), a.y(), epsilon = Self::value_epsilon())
                && self.y() < b.y().max(a.y())
                && self.y() > b.y().min(a.y())
            {
                return true;
            } else {
                return false;
            }
        }

        // Horizontal line
        if abs_diff_eq!(a.y(), b.y(), epsilon = Self::value_epsilon())
            && abs_diff_eq!(self.y(), a.y(), epsilon = Self::value_epsilon())
        {
            if !abs_diff_eq!(self.x(), b.x(), epsilon = Self::value_epsilon())
                && !abs_diff_eq!(self.x(), a.x(), epsilon = Self::value_epsilon())
                && self.x() < b.x().max(a.x())
                && self.x() > b.x().min(a.x())
            {
                return true;
            } else {
                return false;
            }
        }

        // Range check
        if (self.x() < a.x().min(b.x()))
            || (self.x() > a.x().max(b.x()))
            || (self.y() < a.y().min(b.y()))
            || (self.y() > a.y().max(b.y()))
        {
            return false;
        }

        // Exclude end points
        if (abs_diff_eq!(self.x(), a.x(), epsilon = Self::value_epsilon())
            && abs_diff_eq!(self.y(), a.y(), epsilon = Self::value_epsilon()))
            || (abs_diff_eq!(self.x(), b.x(), epsilon = Self::value_epsilon())
                && abs_diff_eq!(self.y(), b.y(), epsilon = Self::value_epsilon()))
        {
            return false;
        }

        let cross = (self.y() - a.y()) * (b.x() - a.x()) - (self.x() - a.x()) * (b.y() - a.y());

        if cross.abs() > Self::epsilon() {
            return false;
        }

        let dot = (self.x() - a.x()) * (b.x() - a.x()) + (self.y() - a.y()) * (b.y() - a.y());

        if dot < Self::Value::zero()
            || abs_diff_eq!(dot, Self::Value::zero(), epsilon = Self::value_epsilon())
        {
            return false;
        }

        let len2 = (b.x() - a.x()) * (b.x() - a.x()) + (b.y() - a.y()) * (b.y() - a.y());

        if dot > len2 || abs_diff_eq!(dot, len2, epsilon = Self::value_epsilon()) {
            return false;
        }

        true
    }

    /// return true if point is inside polygon, false if outside, and None if on perimeter
    fn in_polygon<T: Polygon<Point = Self>>(&self, polygon: &T) -> Option<bool> {
        if polygon.length() < 3 {
            return None;
        }

        let mut inside = false;
        for seg in polygon.iter_segments() {
            let x0 = seg.start().x();
            let y0 = seg.start().y();
            let x1 = seg.end().x();
            let y1 = seg.end().y();

            // on the perimeter of the polygon
            if abs_diff_eq!(x0, self.x(), epsilon = Self::value_epsilon())
                && abs_diff_eq!(y0, self.y(), epsilon = Self::value_epsilon())
            {
                return None;
            }

            if self.on_segment(&seg) {
                return None;
            }

            if abs_diff_eq!(seg.start(), seg.end()) {
                continue;
            }

            let intersect = ((y0 > self.y()) != (y1 > self.y()))
                && (self.x() < (x1 - x0) * (self.y() - y0) / (y1 - y0) + x0);
            if intersect {
                inside = !inside;
            }
        }

        Some(inside)
    }

    /// Returns the distance from the point to the line segment.
    /// Distance is along the normal direction.
    /// Distance is negative if the point is behind the line segment.
    /// If the point is on the line segment, it returns None.
    fn distance_to_segment<T: Segment<Point = Self>>(
        &self,
        segment: &T,
        normal: Self,
        infinite: bool,
    ) -> Option<Self::Value> {
        let normal = normal.normalized();
        let Some(normal) = normal else { return None };
        let dir = Self::from_xy(normal.y(), -normal.x());

        let pdot = self.dot(&dir);
        let s1dot = segment.start().dot(&dir);
        let s2dot = segment.end().dot(&dir);

        let pdotnorm = self.dot(&normal);
        let s1dotnorm = segment.start().dot(&normal);
        let s2dotnorm = segment.end().dot(&normal);

        if !infinite {
            if ((pdot < s1dot || abs_diff_eq!(pdot, s1dot, epsilon = Self::value_epsilon()))
                && (pdot < s2dot || abs_diff_eq!(pdot, s2dot, epsilon = Self::value_epsilon())))
                || ((pdot > s1dot || abs_diff_eq!(pdot, s1dot, epsilon = Self::value_epsilon()))
                    && (pdot > s2dot || abs_diff_eq!(pdot, s2dot, epsilon = Self::value_epsilon())))
            {
                return None; // point doesn't collide with segment, or lies directly on the vertex
            }
            if (abs_diff_eq!(pdot, s1dot, epsilon = Self::value_epsilon())
                && abs_diff_eq!(pdot, s2dot, epsilon = Self::value_epsilon()))
                && (pdotnorm > s1dotnorm && pdotnorm > s2dotnorm)
            {
                return Some((pdotnorm - s1dotnorm).min(pdotnorm - s2dotnorm));
            }
            if (abs_diff_eq!(pdot, s1dot, epsilon = Self::value_epsilon())
                && abs_diff_eq!(pdot, s2dot, epsilon = Self::value_epsilon()))
                && (pdotnorm < s1dotnorm && pdotnorm < s2dotnorm)
            {
                return Some(-(s1dotnorm - pdotnorm).min(s2dotnorm - pdotnorm));
            }
        }

        Some(-(pdotnorm - s1dotnorm + (s1dotnorm - s2dotnorm) * (s1dot - pdot) / (s1dot - s2dot)))
    }
}

mod tests {

    #[test]
    fn test_within_distance() {
        use super::Point2D as _;
        use crate::kernelf64::Point2D;
        let point1 = Point2D { x: 0.0, y: 0.0 };
        let point2 = Point2D { x: 3.0, y: 4.0 };

        assert!(point1.within_distance(&point2, 5.1));
        assert!(!point1.within_distance(&point2, 4.9));

        let point3 = Point2D { x: 1.0, y: 1.0 };
        assert!(point1.within_distance(&point3, 1.5));
        assert!(!point1.within_distance(&point3, 1.4));
    }

    #[test]
    fn test_normalized() {
        use super::Point2D as _;
        use crate::kernelf64::Point2D;
        use approx::abs_diff_eq;
        let point = Point2D { x: 3.0, y: 4.0 };
        let normalized = point.normalized().unwrap();

        assert!(abs_diff_eq!(normalized.x(), 0.6, epsilon = 1e-10));
        assert!(abs_diff_eq!(normalized.y(), 0.8, epsilon = 1e-10));

        let unit_point = Point2D { x: 1.0, y: 0.0 };
        let normalized_unit = unit_point.normalized().unwrap();

        assert!(abs_diff_eq!(normalized_unit.x(), 1.0, epsilon = 1e-10));
        assert!(abs_diff_eq!(normalized_unit.y(), 0.0, epsilon = 1e-10));

        let zero_point = Point2D { x: 0.0, y: 0.0 };
        let normalized_zero = zero_point.normalized();
        assert!(normalized_zero.is_none());
    }

    #[test]
    fn test_on_segment() {
        use super::Point2D as _;
        use crate::kernelf64::Point2D;
        use crate::kernelf64::Segment;
        let a = Point2D { x: 0.0, y: 0.0 };
        let b = Point2D { x: 4.0, y: 4.0 };
        let segment1 = Segment { start: a, end: b };

        // Test point on the segment
        let p1 = Point2D { x: 2.0, y: 2.0 };
        assert!(p1.on_segment(&segment1));

        // Test point not on the segment
        let p2 = Point2D { x: 3.0, y: 2.0 };
        assert!(!p2.on_segment(&segment1));

        // Test point at endpoint (should return false)
        let p3 = Point2D { x: 0.0, y: 0.0 };
        assert!(!p3.on_segment(&segment1));

        // Test vertical line segment
        let c = Point2D { x: 1.0, y: 0.0 };
        let d = Point2D { x: 1.0, y: 4.0 };
        let segment2 = Segment { start: c, end: d };
        let p4 = Point2D { x: 1.0, y: 2.0 };
        assert!(p4.on_segment(&segment2));

        // Test horizontal line segment
        let e = Point2D { x: 0.0, y: 1.0 };
        let f = Point2D { x: 4.0, y: 1.0 };
        let segment3 = Segment { start: e, end: f };
        let p5 = Point2D { x: 2.0, y: 1.0 };
        assert!(p5.on_segment(&segment3));

        // Test point just outside the segment
        let p6 = Point2D { x: 4.1, y: 4.1 };
        assert!(!p6.on_segment(&segment1));
    }

    #[test]
    fn test_point_in_polygon() {
        use super::Point2D as _;
        use crate::kernelf64::{Point2D, Polygon};

        // Create a square polygon
        let square = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: 4.0 },
                Point2D { x: 0.0, y: 4.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        // Test point inside the polygon
        let p1 = Point2D { x: 2.0, y: 2.0 };
        assert_eq!(p1.in_polygon(&square), Some(true));

        // Test point outside the polygon
        let p2 = Point2D { x: 5.0, y: 5.0 };
        assert_eq!(p2.in_polygon(&square), Some(false));

        // Test point on the perimeter
        let p3 = Point2D { x: 0.0, y: 2.0 };
        assert_eq!(p3.in_polygon(&square), None);

        // Test point on a vertex
        let p4 = Point2D { x: 0.0, y: 0.0 };
        assert_eq!(p4.in_polygon(&square), None);

        // Create a triangle polygon
        let triangle = Polygon {
            vertices: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 2.0, y: 4.0 },
            ],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };

        // Test point inside the triangle
        let p5 = Point2D { x: 2.0, y: 1.0 };
        assert_eq!(p5.in_polygon(&triangle), Some(true));

        // Test point outside the triangle
        let p6 = Point2D { x: 3.0, y: 3.0 };
        assert_eq!(p6.in_polygon(&triangle), Some(false));

        // Test point on an edge of the triangle
        let p7 = Point2D { x: 1.0, y: 2.0 };
        assert_eq!(p7.in_polygon(&triangle), None);

        // Test with a polygon that has less than 3 points (should return None)
        let invalid_polygon = Polygon {
            vertices: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 1.0, y: 1.0 }],
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        };
        let p8 = Point2D { x: 0.5, y: 0.5 };
        assert_eq!(p8.in_polygon(&invalid_polygon), None);
    }

    #[test]
    fn test_point_distance_to_segment() {
        use super::Point2D as _;
        use crate::kernelf64::{Point2D, Segment};
        use approx::abs_diff_eq;

        let segment = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 4.0, y: 4.0 },
        };

        let normal = Point2D { x: -1.0, y: 1.0 };

        // Test point on the segment
        let p1 = Point2D { x: 2.0, y: 2.0 };
        let distance1 = p1.distance_to_segment(&segment, normal, false);
        assert!(distance1.is_some());
        assert!(abs_diff_eq!(distance1.unwrap(), 0.0, epsilon = 1e-10));

        // Test point off the segment, but intersecting when extended
        let p2 = Point2D { x: -1.0, y: -1.0 };
        let distance2 = p2.distance_to_segment(&segment, normal, false);
        assert!(distance2.is_none());

        // Test point off the segment, intersecting when extended (infinite = true)
        let distance2_infinite = p2.distance_to_segment(&segment, normal, true);
        assert!(distance2_infinite.is_some());
        assert!(
            abs_diff_eq!(distance2_infinite.unwrap(), 0.0, epsilon = 1e-10),
            "Expected: {}, Got: {}",
            -2.0_f64.sqrt(),
            distance2_infinite.unwrap()
        );

        // Test point not on the line of the segment
        let p3 = Point2D { x: 0.0, y: 2.0 };
        let distance3 = p3.distance_to_segment(&segment, normal, false);
        assert!(distance3.is_some());
        assert!(abs_diff_eq!(
            distance3.unwrap(),
            -2.0_f64.sqrt(),
            epsilon = 1e-10
        ));

        // Test point at the start of the segment
        let p4 = Point2D { x: 0.0, y: 0.0 };
        let distance4 = p4.distance_to_segment(&segment, normal, false);
        assert!(distance4.is_none());

        // Test point at the end of the segment
        let p5 = Point2D { x: 4.0, y: 4.0 };
        let distance5 = p5.distance_to_segment(&segment, normal, false);
        assert!(distance5.is_none());

        // Test with a vertical segment
        let vertical_segment = Segment {
            start: Point2D { x: 2.0, y: 0.0 },
            end: Point2D { x: 2.0, y: 4.0 },
        };
        let vertical_normal = Point2D { x: 1.0, y: 0.0 };

        let p6 = Point2D { x: 3.0, y: 2.0 };
        let distance6 = p6.distance_to_segment(&vertical_segment, vertical_normal, false);
        assert!(distance6.is_some());
        assert!(
            abs_diff_eq!(distance6.unwrap(), -1.0, epsilon = 1e-10),
            "Expected: {}, Got: {}",
            -1.0,
            distance6.unwrap()
        );

        // Test with a horizontal segment
        let horizontal_segment = Segment {
            start: Point2D { x: 0.0, y: 2.0 },
            end: Point2D { x: 4.0, y: 2.0 },
        };
        let horizontal_normal = Point2D { x: 0.0, y: 1.0 };

        let p7 = Point2D { x: 2.0, y: 1.0 };
        let distance7 = p7.distance_to_segment(&horizontal_segment, horizontal_normal, false);
        assert!(distance7.is_some());
        assert!(abs_diff_eq!(distance7.unwrap(), 1.0, epsilon = 1e-10));
    }
}
