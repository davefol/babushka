use approx::{abs_diff_eq, AbsDiffEq};
use num_traits::{Float, One, Zero};

use crate::polygon::Polygon;
use crate::segment::Segment;

pub trait Point2D: Clone {
    type Value: Float + AbsDiffEq + Copy;

    fn x(&self) -> Self::Value;
    fn y(&self) -> Self::Value;

    fn set_x(&mut self, x: Self::Value);
    fn set_y(&mut self, y: Self::Value);

    fn epsilon() -> Self::Value {
        Self::Value::epsilon()
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
            Self::Value::one()
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
        if abs_diff_eq!(a.x(), b.x()) && abs_diff_eq!(self.x(), a.x()) {
            if !abs_diff_eq!(self.y(), b.y())
                && !abs_diff_eq!(self.y(), a.y())
                && self.y() < b.y().max(a.y())
                && self.y() > b.y().min(a.y())
            {
                return true;
            } else {
                return false;
            }
        }

        // Horizontal line
        if abs_diff_eq!(a.y(), b.y()) && abs_diff_eq!(self.y(), a.y()) {
            if !abs_diff_eq!(self.x(), b.x())
                && !abs_diff_eq!(self.x(), a.x())
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
        if (abs_diff_eq!(self.x(), a.x()) && abs_diff_eq!(self.y(), a.y()))
            || (abs_diff_eq!(self.x(), b.x()) && abs_diff_eq!(self.y(), b.y()))
        {
            return false;
        }

        let cross = (self.y() - a.y()) * (b.x() - a.x()) - (self.x() - a.x()) * (b.y() - a.y());

        if cross.abs() > Self::Value::epsilon() {
            return false;
        }

        let dot = (self.x() - a.x()) * (b.x() - a.x()) + (self.y() - a.y()) * (b.y() - a.y());

        if dot < Self::Value::zero() || abs_diff_eq!(dot, Self::Value::zero()) {
            return false;
        }

        let len2 = (b.x() - a.x()) * (b.x() - a.x()) + (b.y() - a.y()) * (b.y() - a.y());

        if dot > len2 || abs_diff_eq!(dot, len2) {
            return false;
        }

        true
    }

    /// return true if point is inside polygon, false if outside, and None if on perimeter
    fn in_polygon<T: Polygon<Point = Self>>(&self, polygon: T) -> Option<bool> {
        if polygon.length() < 3 {
            return None;
        }

        let mut inside = false;
        let offset_x = polygon.offset().x();
        let offset_y = polygon.offset().y();
        for seg in polygon.iter_segments() {
            let x0 = seg.start().x() + offset_x;
            let y0 = seg.start().y() + offset_y;
            let x1 = seg.end().x() + offset_x;
            let y1 = seg.end().y() + offset_y;

            // on the perimeter of the polygon
            if abs_diff_eq!(x0, self.x()) && abs_diff_eq!(y0, self.y()) {
                return None;
            }

            if self.on_segment(&seg) {
                return None;
            }

            if abs_diff_eq!(x0, x1) && abs_diff_eq!(y0, y1) {
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
}

mod tests {
    use super::*;
    use crate::kernelf64::Point2D as Point;
    use crate::kernelf64::Segment;

    #[test]
    fn test_within_distance() {
        let point1 = Point { x: 0.0, y: 0.0 };
        let point2 = Point { x: 3.0, y: 4.0 };

        assert!(point1.within_distance(&point2, 5.1));
        assert!(!point1.within_distance(&point2, 4.9));

        let point3 = Point { x: 1.0, y: 1.0 };
        assert!(point1.within_distance(&point3, 1.5));
        assert!(!point1.within_distance(&point3, 1.4));
    }

    #[test]
    fn test_normalized() {
        let point = Point { x: 3.0, y: 4.0 };
        let normalized = point.normalized().unwrap();

        assert!(abs_diff_eq!(normalized.x(), 0.6, epsilon = 1e-10));
        assert!(abs_diff_eq!(normalized.y(), 0.8, epsilon = 1e-10));

        let unit_point = Point { x: 1.0, y: 0.0 };
        let normalized_unit = unit_point.normalized().unwrap();

        assert!(abs_diff_eq!(normalized_unit.x(), 1.0, epsilon = 1e-10));
        assert!(abs_diff_eq!(normalized_unit.y(), 0.0, epsilon = 1e-10));

        let zero_point = Point { x: 0.0, y: 0.0 };
        let normalized_zero = zero_point.normalized();
        assert!(normalized_zero.is_none());
    }

    #[test]
    fn test_on_segment() {
        let a = Point { x: 0.0, y: 0.0 };
        let b = Point { x: 4.0, y: 4.0 };
        let segment1 = Segment { start: a, end: b };

        // Test point on the segment
        let p1 = Point { x: 2.0, y: 2.0 };
        assert!(p1.on_segment(&segment1));

        // Test point not on the segment
        let p2 = Point { x: 3.0, y: 2.0 };
        assert!(!p2.on_segment(&segment1));

        // Test point at endpoint (should return false)
        let p3 = Point { x: 0.0, y: 0.0 };
        assert!(!p3.on_segment(&segment1));

        // Test vertical line segment
        let c = Point { x: 1.0, y: 0.0 };
        let d = Point { x: 1.0, y: 4.0 };
        let segment2 = Segment { start: c, end: d };
        let p4 = Point { x: 1.0, y: 2.0 };
        assert!(p4.on_segment(&segment2));

        // Test horizontal line segment
        let e = Point { x: 0.0, y: 1.0 };
        let f = Point { x: 4.0, y: 1.0 };
        let segment3 = Segment { start: e, end: f };
        let p5 = Point { x: 2.0, y: 1.0 };
        assert!(p5.on_segment(&segment3));

        // Test point just outside the segment
        let p6 = Point { x: 4.1, y: 4.1 };
        assert!(!p6.on_segment(&segment1));
    }
}
