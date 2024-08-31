use crate::point::Point2D;
use num_traits::Float;

pub trait Segment {
    type Point: Point2D;

    fn start(&self) -> &Self::Point;
    fn end(&self) -> &Self::Point;

    fn intersects_segment(&self, other: &Self, infinite: bool) -> Option<Self::Point> {
        let a = self.start();
        let b = self.end();
        let c = other.start();
        let d = other.end();

        let a1 = b.y() - a.y();
        let b1 = a.x() - b.x();
        let c1 = b.x() * a.y() - a.x() * b.y();
        let a2 = d.y() - c.y();
        let b2 = c.x() - d.x();
        let c2 = d.x() * c.y() - c.x() * d.y();

        let denominator = a1 * b2 - a2 * b1;

        let x = (b1 * c2 - b2 * c1) / denominator;
        let y = (a2 * c1 - a1 * c2) / denominator;

        if !x.is_finite() || !y.is_finite() {
            return None;
        };

        if !infinite {
            if (a.x() - b.x()).abs() > Self::Point::epsilon()
                && if a.x() < b.x() {
                    x < a.x() || x > b.x()
                } else {
                    x > a.x() || x < b.x()
                }
            {
                return None;
            }

            if (a.y() - b.y()).abs() > Self::Point::epsilon()
                && if a.y() < b.y() {
                    y < a.y() || y > b.y()
                } else {
                    y > a.y() || y < b.y()
                }
            {
                return None;
            }

            if (c.x() - d.x()).abs() > Self::Point::epsilon()
                && if c.x() < d.x() {
                    x < c.x() || x > d.x()
                } else {
                    x > c.x() || x < d.x()
                }
            {
                return None;
            }

            if (c.y() - d.y()).abs() > Self::Point::epsilon()
                && if c.y() < d.y() {
                    y < c.y() || y > d.y()
                } else {
                    y > c.y() || y < d.y()
                }
            {
                return None;
            }
        }

        let mut out = a.clone();
        out.set_x(x);
        out.set_y(y);
        Some(out)
    }
}

mod tests {
    #[test]
    fn test_segment_intersects_segment() {
        use super::Segment as _;
        use crate::kernelf64::Point2D;
        use crate::kernelf64::Segment;
        use crate::point::Point2D as _;
        use approx::abs_diff_eq;
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 5.0, y: 5.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 0.0, y: 5.0 },
            end: Point2D { x: 5.0, y: 0.0 },
        };
        let segment3 = Segment {
            start: Point2D { x: 6.0, y: 6.0 },
            end: Point2D { x: 7.0, y: 7.0 },
        };

        // Test intersecting segments
        let intersection = segment1.intersects_segment(&segment2, false);
        assert!(intersection.is_some());
        let point = intersection.unwrap();
        assert!(abs_diff_eq!(point.x(), 2.5));
        assert!(abs_diff_eq!(point.y(), 2.5));

        // Test non-intersecting segments
        let no_intersection = segment1.intersects_segment(&segment3, false);
        assert!(no_intersection.is_none());

        // Test parallel segments
        let segment4 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 6.0, y: 6.0 },
        };
        let parallel_intersection = segment1.intersects_segment(&segment4, false);
        assert!(parallel_intersection.is_none());

        // Test with infinite flag set to true
        let segment5 = Segment {
            start: Point2D { x: 0.0, y: 2.5 },
            end: Point2D { x: 1.5, y: 2.5 },
        };
        let infinite_intersection = segment1.intersects_segment(&segment5, true);
        assert!(infinite_intersection.is_some());
        let infinite_point = infinite_intersection.unwrap();
        assert!(abs_diff_eq!(infinite_point.x(), 2.5));
        assert!(abs_diff_eq!(infinite_point.y(), 2.5));
    }
}
