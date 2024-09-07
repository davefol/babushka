use crate::point::Point2D;
use crate::polygon::Polygon;
use approx::abs_diff_eq;
use num_traits::{Float, One, Zero};
use std::ops::Add;

pub trait Segment: Clone + Copy + Add<Self::Point, Output = Self> {
    type Point: Point2D;

    fn start(&self) -> &Self::Point;
    fn end(&self) -> &Self::Point;

    /// Returns the intersection of this segment with a polygon.
    /// The intersections are ordered by distance from the start of this segment.
    fn intersects_polygon<P>(&self, other: &P) -> Vec<Self::Point>
    where
        P: Polygon<Segment = Self>,
    {
        let mut intersections = vec![];
        for segment in other.iter_segments() {
            let intersection = self.intersects_segment(&segment, false);
            if let Some(intersection) = intersection {
                intersections.push(intersection);
            }
        }
        // order intersections by distance from self.start()
        intersections.sort_by(|a, b| {
            let dist_a = self.start().dot(&(*a - *self.start()));
            let dist_b = self.start().dot(&(*b - *self.start()));
            dist_a.partial_cmp(&dist_b).unwrap()
        });
        intersections
    }

    /// Returns the intersection of this segment with another segment.
    /// Returns None if there is no intersection.
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

        let out = Self::Point::from_xy(x, y);
        Some(out)
    }

    /// Returns the distance from this segment to another segment
    /// if the other segment travelled along direction.
    fn distance_to_segment_along_direction(
        &self,
        other: &Self,
        direction: Self::Point,
    ) -> Option<<<Self as Segment>::Point as Point2D>::Value> {
        let Some(direction) = direction.normalized() else {
            return None;
        };

        let normal = <<Self as Segment>::Point as Point2D>::from_xy(direction.y(), -direction.x());
        let reverse = -direction.clone();

        let a = self.start();
        let b = self.end();
        let e = other.start();
        let f = other.end();

        let dot_a = a.dot(&normal);
        let dot_b = b.dot(&normal);
        let dot_e = e.dot(&normal);
        let dot_f = f.dot(&normal);

        let cross_a = a.dot(&direction);
        let cross_b = b.dot(&direction);
        let cross_e = e.dot(&direction);
        let cross_f = f.dot(&direction);

        let ab_min = dot_a.min(dot_b);
        let ab_max = dot_a.max(dot_b);
        let ef_min = dot_e.min(dot_f);
        let ef_max = dot_e.max(dot_f);

        // segments that will merely touch at one point
        if abs_diff_eq!(ab_max, ef_min) || abs_diff_eq!(ab_min, ef_max) {
            return None;
        }

        // segments miss each other completely
        if ab_max < ef_min || ab_min > ef_max {
            return None;
        }

        let overlap =
            if (ab_max > ef_max && ab_min < ef_min) || (ef_max > ab_max && ef_min < ab_min) {
                One::one()
            } else {
                let min_max = ab_max.min(ef_max);
                let max_min = ab_min.max(ef_min);
                let max_max = ab_max.max(ef_max);
                let min_min = ab_min.min(ef_min);

                (min_max - max_min) / (max_max - min_min)
            };

        let cross_abe = (e.y() - a.y()) * (b.x() - a.x()) - (e.x() - a.x()) * (b.y() - a.y());
        let cross_abf = (f.y() - a.y()) * (b.x() - a.x()) - (f.x() - a.x()) * (b.y() - a.y());

        // lines are colinear
        if abs_diff_eq!(cross_abe, Zero::zero()) && abs_diff_eq!(cross_abf, Zero::zero()) {
            let ab_norm =
                <<Self as Segment>::Point as Point2D>::from_xy(b.y() - a.y(), a.x() - b.x());
            let ef_norm =
                <<Self as Segment>::Point as Point2D>::from_xy(f.y() - e.y(), e.x() - f.x());

            let Some(ab_norm) = ab_norm.normalized() else {
                return None;
            };

            let Some(ef_norm) = ef_norm.normalized() else {
                return None;
            };

            // segment normals must point in opposite directions
            if (ab_norm.y() * ef_norm.x() - ab_norm.x() * ef_norm.y()).abs() < Float::epsilon()
                && ab_norm.y() * ef_norm.y() + ab_norm.x() * ef_norm.x() < Zero::zero()
            {
                // normal of AB segment must point in same direction as given direction vector
                let norm_dot = ab_norm.y() * direction.y() + ab_norm.x() * direction.x();
                if abs_diff_eq!(norm_dot, Zero::zero()) {
                    return None;
                }
                if norm_dot < Zero::zero() {
                    return Some(Zero::zero());
                }
            }
            return None;
        }

        let mut distances = vec![];

        // coincident points
        if abs_diff_eq!(dot_a, dot_e) {
            distances.push(cross_a - cross_e);
        } else if abs_diff_eq!(dot_a, dot_f) {
            distances.push(cross_a - cross_f);
        } else if dot_a > ef_min && dot_a < ef_max {
            let mut d = a.distance_to_segment(other, reverse, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(dhat, Zero::zero()) {
                    let db = b.distance_to_segment(other, reverse, true);
                    if let Some(db) = db {
                        if db < Zero::zero() || abs_diff_eq!(db * overlap, Zero::zero()) {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if abs_diff_eq!(dot_b, dot_e) {
            distances.push(cross_b - cross_e);
        } else if abs_diff_eq!(dot_b, dot_f) {
            distances.push(cross_b - cross_f);
        } else if dot_b > ef_min && dot_b < ef_max {
            let mut d = b.distance_to_segment(other, reverse, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(dhat, Zero::zero()) {
                    let da = a.distance_to_segment(other, reverse, true);
                    if let Some(da) = da {
                        if da < Zero::zero() || abs_diff_eq!(da * overlap, Zero::zero()) {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if dot_e > ab_min && dot_e < ab_max {
            let mut d = e.distance_to_segment(self, direction, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(dhat, Zero::zero()) {
                    let df = f.distance_to_segment(self, direction, true);
                    if let Some(df) = df {
                        if df < Zero::zero() || abs_diff_eq!(df * overlap, Zero::zero()) {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if dot_f > ab_min && dot_f < ab_max {
            let mut d = f.distance_to_segment(self, direction, false);
            if let Some(dhat) = d {
                if abs_diff_eq!(dhat, Zero::zero()) {
                    let de = e.distance_to_segment(self, direction, true);
                    if let Some(de) = de {
                        if de < Zero::zero() || abs_diff_eq!(de * overlap, Zero::zero()) {
                            d = None;
                        }
                    }
                }
            }
            if let Some(d) = d {
                distances.push(d);
            }
        }

        if distances.is_empty() {
            return None;
        }

        return distances
            .into_iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap());
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

    #[test]
    fn test_segment_distance_to_coincident_segment() {
        use super::Segment as _;
        use crate::kernelf64::Point2D;
        use crate::kernelf64::Segment;

        // test segments that will merely touch at one point
        //   | segment 2
        //   |
        // |   segment 1
        // |
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 0.0, y: 1.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 1.0, y: 2.0 },
        };
        let direction = Point2D { x: -1.0, y: 0.0 };
        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        assert_eq!(distance, None);

        // test segments that will miss each other completely
        //   | segment 2
        //   |
        //
        // |   segment 1
        // |
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 0.0, y: 1.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 1.0, y: 2.0 },
            end: Point2D { x: 1.0, y: 3.0 },
        };
        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        assert_eq!(distance, None);

        // test segments that are one away from each other
        //   | segment 2
        // | |
        // | |
        // |   segment 1
        let segment1 = Segment {
            start: Point2D { x: 0.0, y: 0.0 },
            end: Point2D { x: 0.0, y: 2.0 },
        };
        let segment2 = Segment {
            start: Point2D { x: 1.0, y: 1.0 },
            end: Point2D { x: 1.0, y: 3.0 },
        };
        let distance = segment1.distance_to_segment_along_direction(&segment2, direction);
        assert_eq!(distance, Some(1.0));
    }

    #[test]
    fn test_segment_intersects_polygon() {
        use super::Segment as _;
        use crate::kernelf64::{Point2D, Segment, Polygon};

        // Create a square polygon
        let square = Polygon::from(vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 0.0, y: 2.0 },
            Point2D { x: 2.0, y: 2.0 },
            Point2D { x: 2.0, y: 0.0 },
        ]);

        // Create a segment that intersects the square
        let segment = Segment {
            start: Point2D { x: -1.0, y: 1.0 },
            end: Point2D { x: 3.0, y: 1.0 },
        };

        // Get intersections
        let intersections = segment.intersects_polygon(&square);

        // Check if we have the correct number of intersections
        assert_eq!(intersections.len(), 2);

        // Check if the intersections are correct
        assert!(intersections.contains(&Point2D { x: 0.0, y: 1.0 }));
        assert!(intersections.contains(&Point2D { x: 2.0, y: 1.0 }));

        // Create a segment that doesn't intersect the square
        let non_intersecting_segment = Segment {
            start: Point2D { x: -2.0, y: -1.0 },
            end: Point2D { x: -1.0, y: -1.0 },
        };

        // Get intersections
        let no_intersections = non_intersecting_segment.intersects_polygon(&square);

        // Check that there are no intersections
        assert_eq!(no_intersections.len(), 0);
    }
}
